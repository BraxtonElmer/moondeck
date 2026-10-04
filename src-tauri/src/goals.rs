//! Goals and the evidence that proves them.
//!
//! Goals live in `goals.json`. Each one names a kind of proof:
//! - `app_time`: minutes of focus in given apps (from the window tracker)
//! - `git`: at least one commit in a repo that day
//! - `manual`: you tick it yourself
//! Any goal can also be ticked by hand, which counts as proof for that day.

use std::{collections::HashMap, path::Path};

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};
use serde::{Deserialize, Serialize};

use crate::store::{Signal, Store};

const DEFAULT: &str = r#"{
  "goals": [
    {
      "id": "moondeck-commit",
      "name": "Commit to moondeck",
      "due": "22:00",
      "days": "daily",
      "evidence": { "type": "git", "repo": "C:/Users/raxtr/Documents/programming/moondeck" },
      "action": { "label": "Open project", "open": ["Visual Studio Code"] }
    },
    {
      "id": "code-2h",
      "name": "Code for 2 hours",
      "due": "23:00",
      "days": "weekdays",
      "evidence": { "type": "app_time", "apps": ["Code.exe"], "minutes": 120 },
      "action": { "label": "Open VS Code", "open": ["Visual Studio Code"] }
    },
    {
      "id": "read-30",
      "name": "Read 30 min",
      "days": "daily",
      "evidence": { "type": "manual" }
    }
  ]
}
"#;

/// How many past days streaks look back over.
const STREAK_WINDOW: i64 = 60;
/// Days of history dots shown before today.
const HISTORY_DAYS: i64 = 6;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub goals: Vec<Goal>,
}

#[derive(Deserialize, Clone)]
pub struct Goal {
    pub id: String,
    pub name: String,
    /// "HH:MM" local time; no due time means any time today.
    pub due: Option<String>,
    #[serde(default)]
    pub days: Days,
    pub evidence: Evidence,
    pub action: Option<Action>,
}

#[derive(Deserialize, Clone, Default)]
#[serde(untagged)]
pub enum Days {
    #[default]
    Daily,
    Named(String),
    List(Vec<String>),
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Evidence {
    AppTime { apps: Vec<String>, minutes: u32 },
    Git { repo: String },
    Manual,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Action {
    pub label: String,
    #[serde(default)]
    pub open: Vec<String>,
}

#[derive(Serialize)]
pub struct Status {
    pub id: String,
    pub name: String,
    /// Where the proof comes from, e.g. "git", "Window time", "Manual".
    pub source: String,
    /// What the proof says right now, e.g. "20 of 45 min".
    pub detail: String,
    pub due: Option<String>,
    /// Seconds until due; negative when overdue.
    pub due_in: Option<i64>,
    pub pct: u32,
    pub done: bool,
    pub by_hand: bool,
    pub skipped: bool,
    pub manual: bool,
    pub scheduled: bool,
    pub streak: u32,
    /// Oldest first, today excluded. `None` = rest day.
    pub history: Vec<Option<bool>>,
    pub action: Option<Action>,
    pub snoozed: Option<String>,
}

pub fn path(dir: &Path) -> std::path::PathBuf {
    dir.join("goals.json")
}

pub fn load(dir: &Path) -> Result<Config, String> {
    let file = path(dir);
    if !file.exists() {
        std::fs::write(&file, DEFAULT).map_err(|e| e.to_string())?;
    }
    let text = std::fs::read_to_string(&file).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("goals.json: {e}"))
}

// ---------- days ----------

fn day_start(d: NaiveDate) -> i64 {
    Local
        .from_local_datetime(&d.and_time(NaiveTime::MIN))
        .earliest()
        .map(|t| t.timestamp())
        .unwrap_or(0)
}

impl Days {
    fn includes(&self, d: NaiveDate) -> bool {
        let wd = d.weekday();
        let weekend = matches!(wd, Weekday::Sat | Weekday::Sun);
        match self {
            Days::Daily => true,
            Days::Named(n) => match n.to_lowercase().as_str() {
                "weekdays" => !weekend,
                "weekends" => weekend,
                _ => true,
            },
            Days::List(list) => list.iter().any(|x| x.to_lowercase().starts_with(&wd.to_string().to_lowercase()[..3])),
        }
    }
}

// ---------- evidence ----------

/// Per-goal manual marks for one day: the latest of done/undo/skip wins.
fn manual_marks(store: &Store, from: i64, to: i64) -> HashMap<String, String> {
    let mut marks = HashMap::new();
    for s in store.between("manual", from, to).unwrap_or_default() {
        if matches!(s.kind.as_str(), "done" | "undo" | "skip") {
            marks.insert(s.subject, s.kind);
        }
    }
    marks
}

/// Active snoozes: goal id -> label ("1 hour", "Tonight", ...).
fn snoozes(store: &Store, now: i64) -> HashMap<String, String> {
    store
        .between("manual", now - 86_400, now + 1)
        .unwrap_or_default()
        .into_iter()
        .filter(|s| s.kind == "snooze" && s.end.unwrap_or(0) > now)
        .map(|s| (s.subject, s.detail))
        .collect()
}

fn app_secs(store: &Store, apps: &[String], from: i64, to: i64) -> i64 {
    store
        .between("window", from, to)
        .unwrap_or_default()
        .iter()
        .filter(|s| apps.iter().any(|a| s.subject.eq_ignore_ascii_case(a) || s.label.eq_ignore_ascii_case(a)))
        .map(|s| s.end.unwrap_or(s.start).min(to) - s.start.max(from))
        .filter(|d| *d > 0)
        .sum()
}

/// Commit timestamps per local day, one `git log` for the whole streak window.
fn git_days(repo: &str, since: i64) -> HashMap<NaiveDate, u32> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["-C", repo, "log", "--all", "--format=%ct", &format!("--since={since}")]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut days = HashMap::new();
    if let Ok(out) = cmd.output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(t) = line.trim().parse::<i64>().ok().and_then(|t| Local.timestamp_opt(t, 0).single()) {
                *days.entry(t.date_naive()).or_default() += 1;
            }
        }
    }
    days
}

struct Day {
    pct: u32,
    detail: String,
    by_hand: bool,
    skipped: bool,
}

fn evaluate_day(store: &Store, goal: &Goal, date: NaiveDate, git: &HashMap<NaiveDate, u32>, now: i64) -> Day {
    let from = day_start(date);
    let to = day_start(date + Duration::days(1)).min(now + 1);
    let mark = manual_marks(store, from, to).remove(&goal.id);

    let (pct, detail) = match &goal.evidence {
        Evidence::AppTime { apps, minutes } => {
            let mins = app_secs(store, apps, from, to) / 60;
            let pct = if *minutes == 0 { 100 } else { (mins * 100 / *minutes as i64).min(100) as u32 };
            (pct, format!("{mins} of {minutes} min"))
        }
        Evidence::Git { .. } => {
            let n = git.get(&date).copied().unwrap_or(0);
            let detail = match n {
                0 => "No commit yet".to_string(),
                1 => "1 commit".to_string(),
                n => format!("{n} commits"),
            };
            (if n > 0 { 100 } else { 0 }, detail)
        }
        Evidence::Manual => (0, "Not checked yet".to_string()),
    };

    match mark.as_deref() {
        Some("done") if pct < 100 => Day { pct: 100, detail: "Checked by you".into(), by_hand: true, skipped: false },
        Some("skip") => Day { pct, detail: "Skipped today".into(), by_hand: false, skipped: true },
        _ => Day { pct, detail, by_hand: false, skipped: false },
    }
}

// ---------- status ----------

pub fn today(store: &Store, cfg: &Config) -> Vec<Status> {
    let now_dt: DateTime<Local> = Local::now();
    let now = now_dt.timestamp();
    let today = now_dt.date_naive();
    let window_start = day_start(today - Duration::days(STREAK_WINDOW));
    let snoozed = snoozes(store, now);

    cfg.goals
        .iter()
        .map(|g| {
            let git = match &g.evidence {
                Evidence::Git { repo } => git_days(repo, window_start),
                _ => HashMap::new(),
            };
            let day = evaluate_day(store, g, today, &git, now);
            let done = day.pct >= 100;

            // Streak: consecutive scheduled days with proof, counting back from
            // today (if done) or yesterday. Skips and rest days don't break it.
            let mut streak = u32::from(done);
            let mut d = today - Duration::days(1);
            for _ in 0..STREAK_WINDOW {
                if g.days.includes(d) {
                    let past = evaluate_day(store, g, d, &git, now);
                    if past.pct >= 100 {
                        streak += 1;
                    } else if !past.skipped {
                        break;
                    }
                }
                d -= Duration::days(1);
            }

            let history = (1..=HISTORY_DAYS)
                .rev()
                .map(|back| {
                    let d = today - Duration::days(back);
                    g.days.includes(d).then(|| evaluate_day(store, g, d, &git, now).pct >= 100)
                })
                .collect();

            let due_in = g
                .due
                .as_deref()
                .and_then(|t| NaiveTime::parse_from_str(t, "%H:%M").ok())
                .and_then(|t| Local.from_local_datetime(&today.and_time(t)).earliest())
                .map(|t| t.timestamp() - now);

            Status {
                id: g.id.clone(),
                name: g.name.clone(),
                source: match g.evidence {
                    Evidence::AppTime { .. } => "Window time",
                    Evidence::Git { .. } => "git",
                    Evidence::Manual => "Manual",
                }
                .into(),
                detail: day.detail,
                due: g.due.clone(),
                due_in,
                pct: day.pct,
                done,
                by_hand: day.by_hand,
                skipped: day.skipped,
                manual: matches!(g.evidence, Evidence::Manual),
                scheduled: g.days.includes(today),
                streak,
                history,
                action: g.action.clone(),
                snoozed: snoozed.get(&g.id).cloned(),
            }
        })
        .collect()
}

/// Record a manual mark for today: "done", "undo" or "skip".
pub fn mark(store: &Store, id: &str, kind: &str) -> rusqlite::Result<()> {
    store
        .insert(&Signal {
            start: chrono::Utc::now().timestamp(),
            end: None,
            source: "manual".into(),
            kind: kind.into(),
            subject: id.into(),
            label: String::new(),
            detail: String::new(),
        })
        .map(|_| ())
}

/// Hide a goal from nudges until `until` (unix seconds).
pub fn snooze(store: &Store, id: &str, until: i64, label: &str) -> rusqlite::Result<()> {
    store
        .insert(&Signal {
            start: chrono::Utc::now().timestamp(),
            end: Some(until),
            source: "manual".into(),
            kind: "snooze".into(),
            subject: id.into(),
            label: String::new(),
            detail: label.into(),
        })
        .map(|_| ())
}

/// Seconds-from-now for a snooze label.
pub fn snooze_until(label: &str) -> i64 {
    let now = Local::now();
    let at = match label {
        "15 min" => now + Duration::minutes(15),
        "1 hour" => now + Duration::hours(1),
        "Tonight" => {
            let t = now.date_naive().and_time(NaiveTime::from_hms_opt(20, 0, 0).unwrap());
            Local.from_local_datetime(&t).earliest().filter(|t| *t > now).unwrap_or(now + Duration::hours(2))
        }
        _ => now + Duration::hours(1),
    };
    at.timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_parses() {
        let cfg: Config = serde_json::from_str(DEFAULT).unwrap();
        assert_eq!(cfg.goals.len(), 3);
    }

    #[test]
    fn day_filters() {
        let sat = NaiveDate::from_ymd_opt(2026, 10, 3).unwrap();
        let mon = NaiveDate::from_ymd_opt(2026, 10, 5).unwrap();
        assert!(!Days::Named("weekdays".into()).includes(sat));
        assert!(Days::Named("weekdays".into()).includes(mon));
        assert!(Days::List(vec!["Sat".into()]).includes(sat));
        assert!(!Days::List(vec!["sat".into()]).includes(mon));
    }

    #[test]
    fn manual_tick_counts_as_proof() {
        let store = Store::open(Path::new(":memory:")).unwrap();
        let cfg: Config = serde_json::from_str(DEFAULT).unwrap();
        mark(&store, "read-30", "done").unwrap();
        let read = today(&store, &cfg).into_iter().find(|s| s.id == "read-30").unwrap();
        assert!(read.done && read.by_hand);
    }

    #[test]
    fn app_time_counts_focus() {
        let store = Store::open(Path::new(":memory:")).unwrap();
        let now = chrono::Utc::now().timestamp();
        let start = (now - 3600).max(day_start(Local::now().date_naive()));
        store
            .insert(&Signal {
                start,
                end: Some(now),
                source: "window".into(),
                kind: "focus".into(),
                subject: "Code.exe".into(),
                label: "Visual Studio Code".into(),
                detail: String::new(),
            })
            .unwrap();
        let mins = app_secs(&store, &["code.exe".into()], start, now) / 60;
        assert_eq!(mins, (now - start) / 60);
    }
}
