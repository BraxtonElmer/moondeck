//! The deck's launch tiles live in `deck.json` next to the database, so they can be edited by hand.

use std::path::Path;

use serde_json::Value;

const DEFAULT: &str = r#"{
  "start_my_day": {
    "open": ["Visual Studio Code", "Spotify"],
    "focus_minutes": 50
  },
  "launch": [
    { "label": "VS Code", "icon": "code", "open": ["Visual Studio Code"] },
    { "label": "Discord", "icon": "chat", "open": ["Discord"] },
    { "label": "Spotify", "icon": "music", "open": ["Spotify"] },
    { "label": "Genshin", "icon": "pad", "open": ["Genshin Impact"] }
  ]
}
"#;

pub fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("deck.json")
}

/// Read `deck.json`, creating it with defaults on first run.
pub fn load(dir: &Path) -> Result<Value, String> {
    let file = path(dir);
    if !file.exists() {
        std::fs::write(&file, DEFAULT).map_err(|e| e.to_string())?;
    }
    let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("deck.json: {e}"))
}
