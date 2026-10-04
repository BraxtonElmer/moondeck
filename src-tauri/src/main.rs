#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod deck;
mod goals;
mod nudge;
mod store;
#[cfg(windows)]
mod system;
mod timeline;
#[cfg(windows)]
mod tracker;

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use store::Store;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const OVERLAY: &str = "overlay";

/// Bumped on every show, so a stale hide fallback can't close a freshly opened overlay.
static SHOWN: AtomicU64 = AtomicU64::new(0);

/// Stretch the overlay over the monitor the cursor is on, then show it.
fn show(app: &AppHandle) {
    let Some(win) = app.get_webview_window(OVERLAY) else { return };

    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(m) = monitor {
        let _ = win.set_position(*m.position());
        let _ = win.set_size(*m.size());
    }

    SHOWN.fetch_add(1, Ordering::SeqCst);
    let _ = win.show();
    let _ = win.set_focus();
    let _ = win.emit("overlay:shown", ());
}

fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(OVERLAY) {
        let _ = win.hide();
    }
}

/// Let the UI play its exit animation; it calls `hide_overlay` when done.
/// Hides anyway if the UI doesn't answer in time.
fn request_hide(app: &AppHandle) {
    let Some(win) = app.get_webview_window(OVERLAY) else { return };
    if !win.is_visible().unwrap_or(false) {
        return;
    }
    let _ = win.emit("overlay:hide", ());
    let gen = SHOWN.load(Ordering::SeqCst);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        if SHOWN.load(Ordering::SeqCst) == gen {
            hide(&app);
        }
    });
}

/// The overlay animates itself; stop DWM's default show/hide transition on top of it.
#[cfg(windows)]
fn disable_dwm_transitions(win: &tauri::WebviewWindow) {
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED};
    if let Ok(hwnd) = win.hwnd() {
        let on: i32 = 1;
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_TRANSITIONS_FORCEDISABLED,
                &on as *const i32 as _,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }
}

fn toggle(app: &AppHandle) {
    let visible = app
        .get_webview_window(OVERLAY)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if visible { request_hide(app) } else { show(app) }
}

#[tauri::command]
fn hide_overlay(app: AppHandle) {
    hide(&app);
}

/// Windows accent color as `#rrggbb` (stored as 0xAABBGGRR in the registry).
#[tauri::command]
fn accent_color() -> Option<String> {
    #[cfg(windows)]
    {
        let key = windows_registry::CURRENT_USER
            .open(r"Software\Microsoft\Windows\DWM")
            .ok()?;
        let v = key.get_u32("AccentColor").ok()?;
        Some(format!("#{:02x}{:02x}{:02x}", v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff))
    }
    #[cfg(not(windows))]
    None
}

#[tauri::command]
fn timeline(store: tauri::State<Arc<Store>>) -> Result<timeline::Timeline, String> {
    timeline::today(&store).map_err(|e| e.to_string())
}

fn data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

#[tauri::command]
fn deck_config(app: AppHandle) -> Result<serde_json::Value, String> {
    deck::load(&data_dir(&app)?)
}

#[tauri::command]
fn edit_deck(app: AppHandle) -> Result<(), String> {
    let dir = data_dir(&app)?;
    deck::load(&dir)?;
    system::open(&deck::path(&dir).to_string_lossy())
}

#[tauri::command]
fn goals_today(app: AppHandle, store: tauri::State<Arc<Store>>) -> Result<Vec<goals::Status>, String> {
    let cfg = goals::load(&data_dir(&app)?)?;
    Ok(goals::today(&store, &cfg))
}

/// `kind`: "done", "undo" or "skip".
#[tauri::command]
fn mark_goal(store: tauri::State<Arc<Store>>, id: String, kind: String) -> Result<(), String> {
    goals::mark(&store, &id, &kind).map_err(|e| e.to_string())
}

#[tauri::command]
fn snooze_goal(store: tauri::State<Arc<Store>>, id: String, label: String) -> Result<(), String> {
    if label == "Skip today" {
        return goals::mark(&store, &id, "skip").map_err(|e| e.to_string());
    }
    goals::snooze(&store, &id, goals::snooze_until(&label), &label).map_err(|e| e.to_string())
}

#[tauri::command]
fn edit_goals(app: AppHandle) -> Result<(), String> {
    let dir = data_dir(&app)?;
    goals::load(&dir)?;
    system::open(&goals::path(&dir).to_string_lossy())
}

#[tauri::command]
fn open_targets(targets: Vec<String>) -> Result<(), String> {
    let failed: Vec<String> = targets.iter().filter_map(|t| system::open(t).err()).collect();
    if failed.is_empty() { Ok(()) } else { Err(failed.join(", ")) }
}

#[tauri::command]
fn audio_state() -> Result<system::AudioState, String> {
    system::audio_state()
}

#[tauri::command]
fn set_volume(level: u32) -> Result<(), String> {
    system::set_volume(level)
}

#[tauri::command]
fn set_mic_muted(muted: bool) -> Result<(), String> {
    system::set_mic_muted(muted)
}

#[tauri::command]
fn media(action: String) -> Result<(), String> {
    system::media(&action)
}

/// Snip, clipboard history, emoji, lock: get the overlay out of the way, then trigger.
#[tauri::command]
fn shell_action(app: AppHandle, action: String) {
    hide(&app);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(180));
        if let Err(e) = system::shell(&action) {
            eprintln!("{e}");
        }
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed
                        && shortcut.matches(Modifiers::ALT, Code::Space)
                    {
                        toggle(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            hide_overlay,
            accent_color,
            timeline,
            deck_config,
            edit_deck,
            goals_today,
            mark_goal,
            snooze_goal,
            edit_goals,
            open_targets,
            audio_state,
            set_volume,
            set_mic_muted,
            media,
            shell_action
        ])
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let store = Arc::new(Store::open(&dir.join("moondeck.db"))?);
            #[cfg(windows)]
            tracker::spawn(store.clone());
            nudge::spawn(app.handle().clone(), store.clone(), dir.clone());
            app.manage(store);

            let open = MenuItem::with_id(app, "open", "Open  (Alt+Space)", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Moondeck", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;

            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Moondeck")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id.as_ref() {
                    "open" => show(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = e
                    {
                        show(tray.app_handle());
                    }
                })
                .build(app)?;

            app.global_shortcut()
                .register(Shortcut::new(Some(Modifiers::ALT), Code::Space))?;

            #[cfg(windows)]
            if let Some(win) = app.get_webview_window(OVERLAY) {
                disable_dwm_transitions(&win);
            }

            show(app.handle());
            Ok(())
        })
        .on_window_event(|win, event| match event {
            // Overlay behaves like a popup: losing focus sends it back to the tray.
            WindowEvent::Focused(false) => request_hide(win.app_handle()),
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                request_hide(win.app_handle());
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("failed to start moondeck");
}
