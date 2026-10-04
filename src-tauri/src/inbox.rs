//! Shared by every message source: decide whether some text is a task, and by when.
//! Local rules flag candidates; the optional LLM confirms and cleans them up.

use chrono::Local;

use crate::{ai, detect, settings::Settings};

pub struct Classified {
    pub title: String,
    pub due: Option<i64>,
}

/// `text` is everything the rules may look at; `title` is the fallback task title.
pub fn classify(text: &str, title: &str, s: &Settings) -> Option<Classified> {
    let found = detect::detect(text, Local::now())?;
    let mut out = Classified { title: shorten(title, 90), due: found.due };
    if ai::enabled(&s.ai) {
        match ai::extract_task(&s.ai, text, Local::now()) {
            Ok(Some(x)) => {
                out.title = shorten(&x.title, 90);
                out.due = x.due.or(out.due);
            }
            Ok(None) => return None,
            Err(e) => eprintln!("ai: {e}"),
        }
    }
    Some(out)
}

pub fn shorten(s: &str, n: usize) -> String {
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.chars().count() <= n {
        return s;
    }
    let mut out: String = s.chars().take(n - 1).collect();
    out.push('…');
    out
}
