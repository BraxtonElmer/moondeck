//! Speaks up only when proof is missing: a toast an hour before a goal is due,
//! another at 15 minutes, and one when it slips past due.

use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};

use chrono::Local;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::{goals, store::Store};

const CHECK_EVERY: Duration = Duration::from_secs(60);
/// Don't nag about something that went overdue long before Moondeck started.
const OVERDUE_GRACE_S: i64 = 2 * 3600;

pub fn spawn(app: AppHandle, store: Arc<Store>, dir: PathBuf) {
    std::thread::Builder::new()
        .name("nudge".into())
        .spawn(move || {
            let mut sent: HashSet<(String, &'static str)> = HashSet::new();
            let mut day = Local::now().date_naive();
            loop {
                std::thread::sleep(CHECK_EVERY);
                if Local::now().date_naive() != day {
                    day = Local::now().date_naive();
                    sent.clear();
                }
                // Detected tasks with a deadline get the same treatment as goals.
                let now = Local::now().timestamp();
                for t in store.open_tasks().unwrap_or_default() {
                    let Some(due) = t.due else { continue };
                    let Some(stage) = stage(due - now) else { continue };
                    if !sent.insert((format!("task:{}", t.id), stage)) {
                        continue;
                    }
                    let when = chrono::DateTime::from_timestamp(due, 0)
                        .map(|d| d.with_timezone(&Local).format("%H:%M").to_string())
                        .unwrap_or_default();
                    let body = if stage == "overdue" { format!("Was due {when} · {}", t.detail) } else { format!("Due {when} · {}", t.detail) };
                    let _ = app.notification().builder().title(&t.title).body(body).show();
                }

                let Ok(cfg) = goals::load(&dir) else { continue };
                for g in goals::today(&store, &cfg) {
                    if !g.scheduled || g.done || g.skipped || g.snoozed.is_some() {
                        continue;
                    }
                    let (Some(due_in), Some(due)) = (g.due_in, g.due.as_deref()) else { continue };
                    let Some(stage) = stage(due_in) else { continue };
                    if !sent.insert((g.id.clone(), stage)) {
                        continue;
                    }
                    let body = if stage == "overdue" {
                        format!("Past {due} and still no proof · {}", g.detail)
                    } else {
                        format!("Due {due} · {}", g.detail)
                    };
                    let _ = app.notification().builder().title(&g.name).body(body).show();
                }
            }
        })
        .expect("failed to start nudge thread");
}

/// Which reminder a deadline `due_in` seconds away calls for, if any.
fn stage(due_in: i64) -> Option<&'static str> {
    match due_in {
        d if d < -OVERDUE_GRACE_S => None,
        d if d < 0 => Some("overdue"),
        d if d <= 15 * 60 => Some("15m"),
        d if d <= 3600 => Some("1h"),
        _ => None,
    }
}
