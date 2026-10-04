#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const OVERLAY: &str = "overlay";

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

    let _ = win.show();
    let _ = win.set_focus();
    let _ = win.emit("overlay:shown", ());
}

fn hide(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(OVERLAY) {
        let _ = win.hide();
    }
}

fn toggle(app: &AppHandle) {
    let visible = app
        .get_webview_window(OVERLAY)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if visible { hide(app) } else { show(app) }
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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
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
        .invoke_handler(tauri::generate_handler![hide_overlay, accent_color])
        .setup(|app| {
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

            show(app.handle());
            Ok(())
        })
        .on_window_event(|win, event| match event {
            // Overlay behaves like a popup: losing focus sends it back to the tray.
            WindowEvent::Focused(false) => {
                let _ = win.hide();
            }
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = win.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("failed to start moondeck");
}
