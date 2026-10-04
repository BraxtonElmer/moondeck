//! Windows controls the deck can trigger: audio, media keys, launching things, shell shortcuts.

use std::path::{Path, PathBuf};

use serde::Serialize;
use windows::{
    core::PCWSTR,
    Win32::{
        Media::Audio::{
            eCapture, eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
            Shutdown::LockWorkStation,
        },
        UI::{
            Input::KeyboardAndMouse::{
                SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
                VK_LWIN, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_OEM_PERIOD,
            },
            Shell::ShellExecuteW,
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    },
};

type Res<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ---------- audio ----------

#[derive(Serialize)]
pub struct AudioState {
    /// 0–100
    pub volume: u32,
    pub muted: bool,
    pub mic_muted: bool,
}

fn endpoint(capture: bool) -> Res<IAudioEndpointVolume> {
    unsafe {
        // Commands run on pooled threads; make sure COM is up on this one.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let devices: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(err)?;
        let device = devices
            .GetDefaultAudioEndpoint(if capture { eCapture } else { eRender }, eConsole)
            .map_err(err)?;
        device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).map_err(err)
    }
}

pub fn audio_state() -> Res<AudioState> {
    unsafe {
        let out = endpoint(false)?;
        let volume = (out.GetMasterVolumeLevelScalar().map_err(err)? * 100.0).round() as u32;
        let muted = out.GetMute().map_err(err)?.as_bool();
        // No microphone is fine; report it as not muted.
        let mic_muted = endpoint(true)
            .and_then(|m| m.GetMute().map_err(err))
            .map(|b| b.as_bool())
            .unwrap_or(false);
        Ok(AudioState { volume, muted, mic_muted })
    }
}

pub fn set_volume(level: u32) -> Res<()> {
    unsafe {
        let out = endpoint(false)?;
        out.SetMasterVolumeLevelScalar(level.min(100) as f32 / 100.0, std::ptr::null())
            .map_err(err)?;
        if level > 0 {
            out.SetMute(false, std::ptr::null()).map_err(err)?;
        }
        Ok(())
    }
}

pub fn set_mic_muted(muted: bool) -> Res<()> {
    unsafe { endpoint(true)?.SetMute(muted, std::ptr::null()).map_err(err) }
}

// ---------- keys ----------

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Press keys in order, release in reverse.
fn chord(keys: &[VIRTUAL_KEY]) {
    let mut inputs: Vec<INPUT> = keys.iter().map(|&k| key(k, false)).collect();
    inputs.extend(keys.iter().rev().map(|&k| key(k, true)));
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

pub fn media(action: &str) -> Res<()> {
    let vk = match action {
        "playpause" => VK_MEDIA_PLAY_PAUSE,
        "next" => VK_MEDIA_NEXT_TRACK,
        "prev" => VK_MEDIA_PREV_TRACK,
        other => return Err(format!("unknown media action {other}")),
    };
    chord(&[vk]);
    Ok(())
}

/// Shell features that need the overlay out of the way first.
pub fn shell(action: &str) -> Res<()> {
    match action {
        "screenshot" => open("ms-screenclip:"),
        "clipboard" => {
            chord(&[VK_LWIN, VIRTUAL_KEY(b'V' as u16)]);
            Ok(())
        }
        "emoji" => {
            chord(&[VK_LWIN, VK_OEM_PERIOD]);
            Ok(())
        }
        "lock" => unsafe { LockWorkStation().map_err(err) },
        other => Err(format!("unknown action {other}")),
    }
}

// ---------- launching ----------

/// Open a file, folder, URL, exe, or a Start menu app by name ("Spotify", "Visual Studio Code").
pub fn open(target: &str) -> Res<()> {
    let resolved = resolve(target).unwrap_or_else(|| PathBuf::from(target));
    let file = wide(&resolved.to_string_lossy());
    let verb = wide("open");
    let result = unsafe {
        ShellExecuteW(None, PCWSTR(verb.as_ptr()), PCWSTR(file.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL)
    };
    // ShellExecute returns a value > 32 on success.
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!("couldn't open {target}"))
    }
}

/// Plain names are looked up as Start menu shortcuts; anything path- or URL-like is left alone.
pub fn resolve(target: &str) -> Option<PathBuf> {
    if target.contains(['\\', '/', ':']) || target.contains('.') {
        return None;
    }
    let roots = [
        std::env::var("APPDATA").ok().map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs")),
        std::env::var("ProgramData").ok().map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs")),
    ];
    roots.into_iter().flatten().find_map(|root| find_shortcut(&root, target, 3))
}

fn find_shortcut(dir: &Path, name: &str, depth: u8) -> Option<PathBuf> {
    let entries: Vec<_> = std::fs::read_dir(dir).ok()?.flatten().collect();
    let wanted = format!("{name}.lnk");
    if let Some(hit) = entries.iter().find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(&wanted)) {
        return Some(hit.path());
    }
    if depth == 0 {
        return None;
    }
    entries
        .iter()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .find_map(|e| find_shortcut(&e.path(), name, depth - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Depend on this machine's devices and Start menu; run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn reads_audio_state() {
        let s = audio_state().unwrap();
        println!("volume {} muted {} mic_muted {}", s.volume, s.muted, s.mic_muted);
        assert!(s.volume <= 100);
    }

    #[test]
    #[ignore]
    fn resolves_start_menu_apps() {
        for name in ["Spotify", "Visual Studio Code", "Discord", "Genshin Impact"] {
            println!("{name} -> {:?}", resolve(name));
        }
        assert!(resolve("Spotify").is_some());
    }
}
