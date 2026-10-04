//! Non-secret settings in `settings.json`. Keys and passwords go through `secrets`.

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Settings {
    pub autostart: bool,
    pub notifications: bool,
    pub gmail: Gmail,
    pub outlook: Outlook,
    pub hoyolab: Hoyolab,
    pub ai: Ai,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Hoyolab {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Gmail {
    pub enabled: bool,
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Outlook {
    pub enabled: bool,
    /// Application (client) ID of the user's own Azure app registration.
    pub client_id: String,
    pub account: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Ai {
    /// "off", "groq", "gemini" or "custom" (any OpenAI-compatible endpoint, e.g. Ollama).
    pub provider: String,
    pub base_url: String,
    pub model: String,
}

impl Default for Ai {
    fn default() -> Self {
        Self { provider: "off".into(), base_url: String::new(), model: String::new() }
    }
}

fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("settings.json")
}

pub fn load(dir: &Path) -> Settings {
    std::fs::read_to_string(path(dir))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(dir: &Path, s: &Settings) -> Result<(), String> {
    let text = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(path(dir), text).map_err(|e| e.to_string())
}
