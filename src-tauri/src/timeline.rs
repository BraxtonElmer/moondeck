//! Turns raw focus spans into a readable "where did today go" list.

use std::collections::HashMap;

use chrono::{Local, NaiveTime};
use serde::Serialize;

use crate::store::{Signal, Store};

/// Spans of the same app this close together merge into one entry.
const MERGE_GAP_S: i64 = 120;
/// Entries shorter than this are dropped from the list (they still count in totals).
const MIN_ENTRY_S: i64 = 60;

#[derive(Serialize)]
pub struct Entry {
    pub start: i64,
    pub app: String,
    pub what: String,
    pub secs: i64,
    pub away: bool,
}

#[derive(Serialize)]
pub struct Timeline {
    /// Newest first.
    pub entries: Vec<Entry>,
    pub focused: i64,
    pub away: i64,
}

pub fn start_of_today() -> i64 {
    Local::now()
        .date_naive()
        .and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .earliest()
        .map(|t| t.timestamp())
        .unwrap_or(0)
}

pub fn today(store: &Store) -> rusqlite::Result<Timeline> {
    let from = start_of_today();
    let to = chrono::Utc::now().timestamp();
    let clip = |s: &Signal| {
        let start = s.start.max(from);
        let end = s.end.unwrap_or(s.start).min(to).max(start);
        (start, end)
    };

    let mut entries = Vec::new();
    let mut focused = 0;

    // Merge consecutive spans of the same app; remember the title it spent longest on.
    let mut cur: Option<(Entry, i64, HashMap<String, i64>)> = None;
    let flush = |cur: Option<(Entry, i64, HashMap<String, i64>)>, out: &mut Vec<Entry>| {
        if let Some((mut e, _, titles)) = cur {
            if let Some((title, _)) = titles.into_iter().max_by_key(|(_, secs)| *secs) {
                e.what = context(&title, &e.app);
            }
            out.push(e);
        }
    };
    for s in store.between("window", from, to)? {
        let (start, end) = clip(&s);
        let secs = end - start;
        focused += secs;
        match &mut cur {
            Some((e, last_end, titles)) if e.app == s.label && start - *last_end <= MERGE_GAP_S => {
                e.secs += secs;
                *last_end = end;
                *titles.entry(s.detail).or_default() += secs;
            }
            _ => {
                flush(cur.take(), &mut entries);
                let titles = HashMap::from([(s.detail, secs)]);
                cur = Some((Entry { start, app: s.label, what: String::new(), secs, away: false }, end, titles));
            }
        }
    }
    flush(cur, &mut entries);

    let mut away = 0;
    for s in store.between("idle", from, to)? {
        let (start, end) = clip(&s);
        away += end - start;
        entries.push(Entry { start, app: "Away".into(), what: "idle".into(), secs: end - start, away: true });
    }

    entries.retain(|e| e.secs >= MIN_ENTRY_S);
    entries.sort_by_key(|e| std::cmp::Reverse(e.start));
    Ok(Timeline { entries, focused, away })
}

/// Pull the useful bit out of a window title: the project in an editor,
/// the site or page in a browser, the server in Discord.
fn context(title: &str, app: &str) -> String {
    let app = app.to_lowercase();
    let cleaned = title.replace(" — ", " - ").replace(" | ", " - ").replace('●', "");
    let parts: Vec<&str> = cleaned
        .split(" - ")
        .map(str::trim)
        .filter(|p| {
            let p = p.to_lowercase();
            !p.is_empty() && !p.contains(&app) && !app.contains(&p)
        })
        .collect();
    let pick = parts.last().copied().unwrap_or("");
    let mut out: String = pick.chars().take(36).collect();
    if pick.chars().count() > 36 {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::context;

    #[test]
    fn picks_project_from_editor() {
        assert_eq!(context("● main.rs - moondeck - Visual Studio Code", "Visual Studio Code"), "moondeck");
    }

    #[test]
    fn picks_site_from_browser() {
        assert_eq!(context("Pull requests - GitHub — Mozilla Firefox", "Firefox"), "GitHub");
    }

    #[test]
    fn empty_when_title_is_just_the_app() {
        assert_eq!(context("Discord", "Discord"), "");
    }
}
