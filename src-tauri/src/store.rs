//! Local signal store. Everything Moondeck observes lands in one SQLite table.

use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection};

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
             CREATE INDEX IF NOT EXISTS signals_source ON signals(source, kind, start);",
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
}
