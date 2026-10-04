//! The deck: pages of tiles, kept in `deck.json` and edited from the overlay.
//!
//! A tile has a `label`, an `icon` (a built-in icon name, or "app" to use the real
//! app icon) and a `run`:
//! - `{"type": "open", "targets": [...]}`: apps by Start menu name, files, folders, URLs
//! - `{"type": "routine", "targets": [...], "focus": 50}`: open several things and start focus
//! - `{"type": "builtin", "action": "mic"}`: focus, timer, mic, playpause, next, prev,
//!   volume, snip, clipboard, emoji, lock

use std::path::Path;

use serde_json::{json, Value};

fn builtin(label: &str, icon: &str, action: &str) -> Value {
    json!({ "label": label, "icon": icon, "run": { "type": "builtin", "action": action } })
}

fn open(label: &str, targets: &[&str]) -> Value {
    json!({ "label": label, "icon": "app", "run": { "type": "open", "targets": targets } })
}

fn controls() -> Value {
    json!({ "name": "Controls", "tiles": [
        builtin("Focus", "target", "focus"),
        builtin("Timer", "timer", "timer"),
        builtin("Mic", "mic", "mic"),
        builtin("Play/Pause", "playpause", "playpause"),
        builtin("Next", "next", "next"),
        builtin("Volume", "vol", "volume"),
    ]})
}

fn tools() -> Value {
    json!({ "name": "Tools", "tiles": [
        builtin("Snip", "shot", "snip"),
        builtin("Clipboard", "clip", "clipboard"),
        builtin("Emoji", "smile", "emoji"),
        builtin("Lock PC", "lock", "lock"),
    ]})
}

fn default_deck() -> Value {
    json!({ "pages": [
        { "name": "Launch", "tiles": [
            { "label": "Start my day", "icon": "sun", "run": { "type": "routine", "targets": ["Visual Studio Code", "Spotify"], "focus": 50 } },
            open("VS Code", &["Visual Studio Code"]),
            open("Discord", &["Discord"]),
            open("Spotify", &["Spotify"]),
            open("Genshin", &["Genshin Impact"]),
        ]},
        controls(),
        tools(),
    ]})
}

/// Convert the first `deck.json` format (`start_my_day` + `launch`) to pages.
fn migrate(old: &Value) -> Value {
    let mut launch = Vec::new();
    if let Some(day) = old.get("start_my_day") {
        launch.push(json!({
            "label": "Start my day", "icon": "sun",
            "run": { "type": "routine", "targets": day["open"].clone(), "focus": day["focus_minutes"].as_u64().unwrap_or(50) }
        }));
    }
    for t in old["launch"].as_array().cloned().unwrap_or_default() {
        launch.push(json!({ "label": t["label"], "icon": "app", "run": { "type": "open", "targets": t["open"].clone() } }));
    }
    json!({ "pages": [ { "name": "Launch", "tiles": launch }, controls(), tools() ] })
}

pub fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("deck.json")
}

/// Read `deck.json`, creating or upgrading it as needed.
pub fn load(dir: &Path) -> Result<Value, String> {
    let file = path(dir);
    if !file.exists() {
        save(dir, &default_deck())?;
    }
    let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("deck.json: {e}"))?;
    if v.get("pages").is_none() {
        let upgraded = migrate(&v);
        save(dir, &upgraded)?;
        return Ok(upgraded);
    }
    Ok(v)
}

pub fn save(dir: &Path, deck: &Value) -> Result<(), String> {
    if !deck["pages"].is_array() {
        return Err("deck needs a pages list".into());
    }
    let text = serde_json::to_string_pretty(deck).map_err(|e| e.to_string())?;
    std::fs::write(path(dir), text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_first_format() {
        let old = json!({
            "start_my_day": { "open": ["Spotify"], "focus_minutes": 25 },
            "launch": [{ "label": "VS Code", "icon": "code", "open": ["Visual Studio Code"] }]
        });
        let v = migrate(&old);
        let launch = &v["pages"][0]["tiles"];
        assert_eq!(launch[0]["run"]["type"], "routine");
        assert_eq!(launch[0]["run"]["focus"], 25);
        assert_eq!(launch[1]["run"]["targets"][0], "Visual Studio Code");
        assert_eq!(v["pages"].as_array().unwrap().len(), 3);
    }
}
