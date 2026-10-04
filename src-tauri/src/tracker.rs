//! Foreground-window tracker. Records which app has focus as `window/focus` spans
//! and time away from the keyboard (idle or locked) as `idle/away` spans.
//!
//! Runs on its own thread: a WinEvent hook fires on every focus change, and a
//! 15 s timer catches title changes, idle time, and keeps the open span's end fresh
//! so a crash loses at most one tick.

use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    sync::Arc,
};

use windows::{
    core::{BOOL, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM},
        Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
        System::{
            SystemInformation::GetTickCount,
            Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::{
            Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
            WindowsAndMessaging::{
                DispatchMessageW, EnumChildWindows, GetForegroundWindow, GetMessageW, GetWindowTextW,
                GetWindowThreadProcessId, SetTimer, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
                WINEVENT_SKIPOWNPROCESS, WM_TIMER,
            },
        },
    },
};

use crate::store::{Signal, Store};

const TICK_MS: u32 = 15_000;
/// No input for this long counts as away.
const AWAY_AFTER_S: i64 = 300;

struct Open {
    id: i64,
    exe: String,
    title: String,
}

struct Tracker {
    store: Arc<Store>,
    open: Option<Open>,
    away_since: Option<i64>,
    names: HashMap<String, String>,
}

thread_local! {
    static TRACKER: RefCell<Option<Tracker>> = const { RefCell::new(None) };
}

fn with(f: impl FnOnce(&mut Tracker)) {
    TRACKER.with(|t| {
        if let Some(t) = t.borrow_mut().as_mut() {
            f(t)
        }
    });
}

pub fn spawn(store: Arc<Store>) {
    std::thread::Builder::new()
        .name("tracker".into())
        .spawn(move || unsafe {
            TRACKER.with(|t| {
                *t.borrow_mut() = Some(Tracker { store, open: None, away_since: None, names: HashMap::new() })
            });
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(on_foreground),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );
            SetTimer(None, 1, TICK_MS, None);
            with(|t| t.focus_changed());

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if msg.message == WM_TIMER {
                    with(|t| t.tick());
                }
                DispatchMessageW(&msg);
            }
            let _ = UnhookWinEvent(hook);
        })
        .expect("failed to start tracker thread");
}

unsafe extern "system" fn on_foreground(_: HWINEVENTHOOK, _: u32, _: HWND, _: i32, _: i32, _: u32, _: u32) {
    with(|t| t.focus_changed());
}

impl Tracker {
    fn focus_changed(&mut self) {
        if self.away_since.is_some() {
            return; // tick() decides when we're back
        }
        if let Some((exe, title)) = foreground() {
            self.switch_to(exe, title, now());
        }
    }

    fn tick(&mut self) {
        let now = now();
        let idle = idle_secs();
        let fg = foreground();
        let locked = fg.as_ref().is_some_and(|(exe, _)| is_lock_screen(exe));

        match self.away_since {
            None if idle >= AWAY_AFTER_S || locked => {
                let since = if locked { now } else { now - idle };
                self.close(since);
                self.away_since = Some(since);
            }
            Some(since) if idle < AWAY_AFTER_S && !locked => {
                let back = now - idle;
                let _ = self.store.insert(&Signal {
                    start: since,
                    end: Some(back),
                    source: "idle".into(),
                    kind: "away".into(),
                    subject: "away".into(),
                    label: "Away".into(),
                    detail: String::new(),
                });
                self.away_since = None;
                if let Some((exe, title)) = fg {
                    self.switch_to(exe, title, back);
                }
            }
            None => {
                if let Some(o) = &self.open {
                    let _ = self.store.set_end(o.id, now);
                }
                if let Some((exe, title)) = fg {
                    self.switch_to(exe, title, now);
                }
            }
            Some(_) => {}
        }
    }

    fn switch_to(&mut self, exe: String, title: String, at: i64) {
        if is_own(&exe) {
            return;
        }
        if is_lock_screen(&exe) {
            self.close(at);
            self.away_since = Some(at);
            return;
        }
        if let Some(o) = &self.open {
            if o.exe == exe && o.title == title {
                return;
            }
        }
        self.close(at);

        let label = self.friendly(&exe);
        let signal = Signal {
            start: at,
            end: Some(at),
            source: "window".into(),
            kind: "focus".into(),
            subject: file_name(&exe).to_string(),
            label,
            detail: title.clone(),
        };
        if let Ok(id) = self.store.insert(&signal) {
            self.open = Some(Open { id, exe, title });
        }
    }

    fn close(&mut self, at: i64) {
        if let Some(o) = self.open.take() {
            let _ = self.store.set_end(o.id, at);
        }
    }

    fn friendly(&mut self, exe: &str) -> String {
        self.names
            .entry(exe.to_string())
            .or_insert_with(|| {
                unsafe { file_description(exe) }.unwrap_or_else(|| {
                    let name = file_name(exe);
                    name.strip_suffix(".exe").or(name.strip_suffix(".EXE")).unwrap_or(name).to_string()
                })
            })
            .clone()
    }
}

// ---------- helpers ----------

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

fn is_own(exe: &str) -> bool {
    file_name(exe).eq_ignore_ascii_case("moondeck.exe")
}

fn is_lock_screen(exe: &str) -> bool {
    file_name(exe).eq_ignore_ascii_case("LockApp.exe")
}

fn idle_secs() -> i64 {
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe {
        if !GetLastInputInfo(&mut info).as_bool() {
            return 0;
        }
        (GetTickCount().wrapping_sub(info.dwTime) / 1000) as i64
    }
}

/// Executable path and title of the focused window.
fn foreground() -> Option<(String, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(hwnd, &mut buf);
        let title = String::from_utf16_lossy(&buf[..n.max(0) as usize]);

        let mut pid = window_pid(hwnd);
        let mut exe = process_path(pid)?;
        // Store apps are hosted by ApplicationFrameHost; the real app is a child window.
        if file_name(&exe).eq_ignore_ascii_case("ApplicationFrameHost.exe") {
            let mut found = (pid, 0u32);
            let _ = EnumChildWindows(Some(hwnd), Some(find_child), LPARAM(&mut found as *mut _ as isize));
            if found.1 != 0 {
                pid = found.1;
                exe = process_path(pid)?;
            }
        }
        Some((exe, title))
    }
}

unsafe extern "system" fn find_child(hwnd: HWND, lp: LPARAM) -> BOOL {
    let state = &mut *(lp.0 as *mut (u32, u32));
    let pid = window_pid(hwnd);
    if pid != state.0 {
        state.1 = pid;
        return BOOL(0);
    }
    BOOL(1)
}

fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(handle);
        res.ok()?;
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The `FileDescription` from an exe's version info, e.g. "Visual Studio Code".
unsafe fn file_description(exe: &str) -> Option<String> {
    let path = wide(exe);
    let size = GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None);
    if size == 0 {
        return None;
    }
    let mut data = vec![0u8; size as usize];
    GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, data.as_mut_ptr() as *mut c_void).ok()?;

    let mut ptr: *mut c_void = std::ptr::null_mut();
    let mut len = 0u32;
    let q = wide("\\VarFileInfo\\Translation");
    if !VerQueryValueW(data.as_ptr() as *const c_void, PCWSTR(q.as_ptr()), &mut ptr, &mut len).as_bool() || len < 4 {
        return None;
    }
    let lang = *(ptr as *const u16);
    let cp = *(ptr as *const u16).add(1);

    let q = wide(&format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileDescription"));
    if !VerQueryValueW(data.as_ptr() as *const c_void, PCWSTR(q.as_ptr()), &mut ptr, &mut len).as_bool() || len == 0 {
        return None;
    }
    let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr as *const u16, len as usize));
    let text = text.trim_end_matches('\0').trim();
    (!text.is_empty()).then(|| text.to_string())
}
