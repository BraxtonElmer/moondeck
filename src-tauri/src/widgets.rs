//! Pinned widgets: small always-on-top windows (Up next, Timer, Notes) that stay out
//! while you work. Which ones are open, and where, lives in `widgets.json`.

use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub const PREFIX: &str = "widget-";

/// (kind, width, height) in logical pixels.
const KINDS: &[(&str, f64, f64)] = &[("next", 300.0, 150.0), ("timer", 260.0, 150.0), ("notes", 280.0, 270.0)];

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Placement {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub open: bool,
}

type Saved = BTreeMap<String, Placement>;

fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("widgets.json")
}

fn load(dir: &Path) -> Saved {
    std::fs::read_to_string(path(dir)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

fn save(dir: &Path, saved: &Saved) {
    if let Ok(text) = serde_json::to_string_pretty(saved) {
        let _ = std::fs::write(path(dir), text);
    }
}

fn data_dir(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok()
}

pub fn open(app: &AppHandle, kind: &str) -> Result<(), String> {
    let &(_, w, h) = KINDS.iter().find(|(k, ..)| *k == kind).ok_or(format!("unknown widget {kind}"))?;
    let label = format!("{PREFIX}{kind}");
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    let dir = data_dir(app).ok_or("no data dir")?;
    let mut saved = load(&dir);
    let place = saved.entry(kind.to_string()).or_default();

    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("widget.html".into()))
        .title("Moondeck")
        .inner_size(w, h)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false);

    // First time: stack down the right edge of the primary monitor.
    let index = KINDS.iter().position(|(k, ..)| *k == kind).unwrap_or(0) as f64;
    let (x, y) = match (place.x, place.y) {
        (Some(x), Some(y)) => (x as f64, y as f64),
        _ => {
            let m = app.primary_monitor().ok().flatten();
            let scale = m.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);
            let width = m.as_ref().map(|m| m.size().width as f64).unwrap_or(1920.0);
            (width - (w + 24.0) * scale, (96.0 + index * 170.0) * scale)
        }
    };
    let win = builder.build().map_err(|e| e.to_string())?;
    let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
    let _ = win.show();

    place.open = true;
    save(&dir, &saved);
    Ok(())
}

pub fn close(app: &AppHandle, kind: &str) {
    if let Some(win) = app.get_webview_window(&format!("{PREFIX}{kind}")) {
        let _ = win.close();
    }
    if let Some(dir) = data_dir(app) {
        let mut saved = load(&dir);
        saved.entry(kind.to_string()).or_default().open = false;
        save(&dir, &saved);
    }
}

/// Remember where a widget was dragged to.
pub fn moved(app: &AppHandle, label: &str, pos: PhysicalPosition<i32>) {
    let Some(kind) = label.strip_prefix(PREFIX) else { return };
    let Some(dir) = data_dir(app) else { return };
    let mut saved = load(&dir);
    let place = saved.entry(kind.to_string()).or_default();
    place.x = Some(pos.x);
    place.y = Some(pos.y);
    save(&dir, &saved);
}

/// Reopen whatever was pinned when Moondeck last quit.
pub fn restore(app: &AppHandle) {
    let Some(dir) = data_dir(app) else { return };
    for (kind, place) in load(&dir) {
        if place.open {
            let _ = open(app, &kind);
        }
    }
}
