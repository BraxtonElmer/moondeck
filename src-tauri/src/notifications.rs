//! Windows notifications as a signal source. Windows keeps recent toasts in a local
//! SQLite database and prunes them once dismissed, so this polls it read-only and keeps
//! its own copy. Anything that reads like a task becomes an inbox task.

use std::{path::PathBuf, sync::Arc, time::Duration};

use chrono::Local;
use rusqlite::{Connection, OpenFlags};

use crate::{
    ai, detect, settings,
    store::{NewTask, Signal, Store},
};

const POLL: Duration = Duration::from_secs(30);
const STATE_KEY: &str = "notifications.order";

fn db_path() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Notifications\wpndatabase.db"))
}

pub fn spawn(store: Arc<Store>, dir: PathBuf) {
    std::thread::Builder::new()
        .name("notifications".into())
        .spawn(move || loop {
            let s = settings::load(&dir);
            if s.notifications {
                if let Err(e) = poll(&store, &s) {
                    eprintln!("notifications: {e}");
                }
            }
            std::thread::sleep(POLL);
        })
        .expect("failed to start notifications thread");
}

struct Toast {
    order: i64,
    id: i64,
    handler: String,
    arrived: i64,
    texts: Vec<String>,
}

fn poll(store: &Store, s: &settings::Settings) -> Result<(), String> {
    let path = db_path().ok_or("no LOCALAPPDATA")?;
    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)
        .map_err(|e| e.to_string())?;

    let mut last: i64 = store.get_state(STATE_KEY).and_then(|v| v.parse().ok()).unwrap_or(0);
    let max: i64 = conn
        .query_row(r#"SELECT COALESCE(MAX("Order"), 0) FROM Notification"#, [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if max < last {
        last = 0; // Windows reset its database
    }

    let mut stmt = conn
        .prepare(
            r#"SELECT n."Order", n.Id, h.PrimaryId, n.ArrivalTime, n.Payload
               FROM Notification n JOIN NotificationHandler h ON h.RecordId = n.HandlerId
               WHERE n.Type = 'toast' AND n."Order" > ?1 ORDER BY n."Order""#,
        )
        .map_err(|e| e.to_string())?;
    let toasts: Vec<Toast> = stmt
        .query_map([last], |r| {
            let payload: Vec<u8> = r.get(4)?;
            Ok(Toast {
                order: r.get(0)?,
                id: r.get(1)?,
                handler: r.get(2)?,
                arrived: filetime_to_unix(r.get(3)?),
                texts: texts(&String::from_utf8_lossy(&payload)),
            })
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();

    for t in toasts {
        last = last.max(t.order);
        if t.texts.is_empty() || is_own(&t.handler) {
            continue;
        }
        let app = app_name(&t.handler);
        let _ = store.insert(&Signal {
            start: t.arrived,
            end: None,
            source: "notification".into(),
            kind: "toast".into(),
            subject: t.handler.clone(),
            label: app.clone(),
            detail: t.texts.join("\n"),
        });
        if let Some(task) = to_task(&t, &app, s) {
            let _ = store.add_task(&task);
        }
    }
    store.set_state(STATE_KEY, &last.to_string()).map_err(|e| e.to_string())
}

/// Moondeck's own nudges. Dev builds send toasts under PowerShell's app id.
fn is_own(handler: &str) -> bool {
    let h = handler.to_lowercase();
    h.contains("moondeck") || h.ends_with("powershell.exe")
}

fn to_task(t: &Toast, app: &str, s: &settings::Settings) -> Option<NewTask> {
    let text = t.texts.join(". ");
    let found = detect::detect(&text, Local::now())?;
    // First line is usually the sender or headline; the rest is the message.
    let (head, body) = match t.texts.as_slice() {
        [only] => (String::new(), only.clone()),
        [head, rest @ ..] => (head.clone(), rest.join(" ")),
        [] => return None,
    };
    let mut title = shorten(&body, 90);
    let mut due = found.due;

    // The LLM, if configured, gets the final say on whether this is a task and when it's due.
    if ai::enabled(&s.ai) {
        match ai::extract_task(&s.ai, &text, Local::now()) {
            Ok(Some(x)) => {
                title = shorten(&x.title, 90);
                due = x.due.or(due);
            }
            Ok(None) => return None,
            Err(e) => eprintln!("ai: {e}"),
        }
    }
    Some(NewTask {
        created: t.arrived,
        source: "notification".into(),
        reference: format!("{}:{}", t.id, t.arrived),
        app: app.to_string(),
        title,
        detail: if head.is_empty() { format!("via {app}") } else { format!("{head} · via {app}") },
        due,
    })
}

fn shorten(s: &str, n: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut out: String = s.chars().take(n - 1).collect();
    out.push('…');
    out
}

/// Windows FILETIME (100 ns ticks since 1601) to unix seconds.
fn filetime_to_unix(ft: i64) -> i64 {
    ft / 10_000_000 - 11_644_473_600
}

/// Text content of every `<text>` element in a toast payload, in order.
fn texts(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<text") {
        rest = &rest[start..];
        let Some(open_end) = rest.find('>') else { break };
        if rest[..open_end].ends_with('/') {
            rest = &rest[open_end + 1..];
            continue;
        }
        let body = &rest[open_end + 1..];
        let Some(close) = body.find("</text>") else { break };
        let text = unescape(body[..close].trim());
        if !text.is_empty() {
            out.push(text);
        }
        rest = &body[close..];
    }
    out
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(end) = rest.find(';').filter(|e| *e < 10) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let ent = &rest[1..end];
        let ch = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if ent.starts_with("#x") => u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32),
            _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match ch {
            Some(c) => out.push(c),
            None => out.push_str(&rest[..=end]),
        }
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

/// "5319275A.WhatsAppDesktop_cv1g1gvanyjgm!App" -> "WhatsApp", "com.squirrel.Discord.Discord" -> "Discord".
fn app_name(handler: &str) -> String {
    let (pkg, entry) = handler.split_once('!').unwrap_or((handler, ""));
    if !entry.is_empty() && !entry.eq_ignore_ascii_case("app") {
        return entry.to_string();
    }
    let pkg = pkg.split('_').next().unwrap_or(pkg);
    let pkg = pkg.rsplit(['\\', '/']).next().unwrap_or(pkg).trim_end_matches(".exe");
    let last = pkg.rsplit('.').next().unwrap_or(pkg);
    last.strip_suffix("Desktop").filter(|s| !s.is_empty()).unwrap_or(last).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_toast_text() {
        let xml = r#"<toast><visual><binding template="ToastGeneric"><text>Prof. Lee</text><text>Essay due Fri &amp; slides by 5pm</text><text hint-style="x"/></binding></visual></toast>"#;
        assert_eq!(texts(xml), vec!["Prof. Lee", "Essay due Fri & slides by 5pm"]);
    }

    #[test]
    fn names_apps() {
        assert_eq!(app_name("5319275A.WhatsAppDesktop_cv1g1gvanyjgm!App"), "WhatsApp");
        assert_eq!(app_name("com.squirrel.Discord.Discord"), "Discord");
        assert_eq!(app_name("Claude_pzs8sxrjxfjjc!Claude"), "Claude");
        assert_eq!(app_name("MSTeams_8wekyb3d8bbwe!MSTeams"), "MSTeams");
    }

    #[test]
    fn filetime() {
        assert_eq!(filetime_to_unix(116_444_736_000_000_000), 0);
    }
}
