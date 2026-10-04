//! Local signal store. Everything Moondeck observes lands in one SQLite table.

use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

/// A piece of evidence. Spans (window focus, away time) have an `end`; point events don't.
#[derive(Debug, Clone)]
pub struct Signal {
    pub start: i64,
    pub end: Option<i64>,
    pub source: String,
    pub kind: String,
    /// Machine id of the thing observed, e.g. `Code.exe` or a repo path.
    pub subject: String,
    /// Human name for display, e.g. `Visual Studio Code`.
    pub label: String,
    pub detail: String,
}

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS signals (
                 id      INTEGER PRIMARY KEY,
                 start   INTEGER NOT NULL,
                 end     INTEGER,
                 source  TEXT NOT NULL,
                 kind    TEXT NOT NULL,
                 subject TEXT NOT NULL,
                 label   TEXT NOT NULL DEFAULT '',
                 detail  TEXT NOT NULL DEFAULT ''
             );
             CREATE INDEX IF NOT EXISTS signals_start ON signals(start);
             CREATE INDEX IF NOT EXISTS signals_source ON signals(source, kind, start);
             CREATE TABLE IF NOT EXISTS state (
                 key   TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS tasks (
                 id      INTEGER PRIMARY KEY,
                 created INTEGER NOT NULL,
                 source  TEXT NOT NULL,
                 ref     TEXT NOT NULL,
                 app     TEXT NOT NULL DEFAULT '',
                 title   TEXT NOT NULL,
                 detail  TEXT NOT NULL DEFAULT '',
                 due     INTEGER,
                 status  TEXT NOT NULL DEFAULT 'open',
                 UNIQUE (source, ref)
             );",
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Insert a signal and return its row id (used to extend open spans).
    pub fn insert(&self, s: &Signal) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO signals (start, end, source, kind, subject, label, detail) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![s.start, s.end, s.source, s.kind, s.subject, s.label, s.detail],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn set_end(&self, id: i64, end: i64) -> rusqlite::Result<()> {
        self.conn
            .lock()
            .unwrap()
            .execute("UPDATE signals SET end = ?2 WHERE id = ?1", params![id, end])?;
        Ok(())
    }

    /// Signals from one source that overlap `[from, to)`, oldest first.
    pub fn between(&self, source: &str, from: i64, to: i64) -> rusqlite::Result<Vec<Signal>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT start, end, source, kind, subject, label, detail FROM signals
             WHERE source = ?1 AND start < ?3 AND COALESCE(end, start) >= ?2
             ORDER BY start",
        )?;
        let rows = stmt.query_map(params![source, from, to], |r| {
            Ok(Signal {
                start: r.get(0)?,
                end: r.get(1)?,
                source: r.get(2)?,
                kind: r.get(3)?,
                subject: r.get(4)?,
                label: r.get(5)?,
                detail: r.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_state(&self, key: &str) -> Option<String> {
        self.conn
            .lock()
            .unwrap()
            .query_row("SELECT value FROM state WHERE key = ?1", [key], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
    }

    pub fn set_state(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO state (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    /// Add a detected task; returns false if this source/ref was already seen.
    pub fn add_task(&self, t: &NewTask) -> rusqlite::Result<bool> {
        let n = self.conn.lock().unwrap().execute(
            "INSERT OR IGNORE INTO tasks (created, source, ref, app, title, detail, due) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![t.created, t.source, t.reference, t.app, t.title, t.detail, t.due],
        )?;
        Ok(n > 0)
    }

    /// Open tasks, soonest due first (undated last), newest first within that.
    pub fn open_tasks(&self) -> rusqlite::Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT id, created, source, app, title, detail, due FROM tasks WHERE status = 'open'
             ORDER BY due IS NULL, due, created DESC LIMIT 100",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Task {
                id: r.get(0)?,
                created: r.get(1)?,
                source: r.get(2)?,
                app: r.get(3)?,
                title: r.get(4)?,
                detail: r.get(5)?,
                due: r.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn set_task_status(&self, id: i64, status: &str) -> rusqlite::Result<()> {
        self.conn
            .lock()
            .unwrap()
            .execute("UPDATE tasks SET status = ?2 WHERE id = ?1", params![id, status])?;
        Ok(())
    }
}

pub struct NewTask {
    pub created: i64,
    pub source: String,
    /// Source-specific id used to avoid duplicates (notification id, message id).
    pub reference: String,
    pub app: String,
    pub title: String,
    pub detail: String,
    pub due: Option<i64>,
}

#[derive(Serialize, Clone)]
pub struct Task {
    pub id: i64,
    pub created: i64,
    pub source: String,
    pub app: String,
    pub title: String,
    pub detail: String,
    pub due: Option<i64>,
}
