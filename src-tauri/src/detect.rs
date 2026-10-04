//! Local rules that spot tasks and deadlines in short text: notifications, email subjects.
//! Cheap, private, offline. The optional LLM only ever refines what these rules flag.

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};

/// Words that make text a task on their own.
const STRONG: &[&str] = &[
    "deadline", "due", "reminder", "remind me", "don't forget", "dont forget", "submit", "submission",
    "assignment", "homework", "invoice", "payment", "pay by", "renew", "renewal", "expires", "expiring",
    "action required", "rsvp", "interview", "appointment", "exam", "todo", "to-do", "overdue",
];
/// Words that only count when there's also a date or time.
const WEAK: &[&str] = &[
    "meeting", "call", "review", "sign", "confirm", "deliver", "send", "finish", "complete", "book",
    "by", "before", "until", "need", "please", "can you", "could you", "starts", "start",
];

#[derive(Debug, PartialEq)]
pub struct Found {
    pub due: Option<i64>,
    /// The keyword that triggered it, for the UI ("due", "meeting").
    pub reason: String,
}

pub fn detect(text: &str, now: DateTime<Local>) -> Option<Found> {
    let t = text.to_lowercase().replace('’', "'");
    let due = parse_due(&t, now);
    if let Some(k) = STRONG.iter().find(|k| has_phrase(&t, k)) {
        return Some(Found { due, reason: k.to_string() });
    }
    if due.is_some() {
        if let Some(k) = WEAK.iter().find(|k| has_phrase(&t, k)) {
            return Some(Found { due, reason: k.to_string() });
        }
    }
    None
}

/// `phrase` appears in `text` on word boundaries.
fn has_phrase(text: &str, phrase: &str) -> bool {
    text.match_indices(phrase).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + phrase.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

fn tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == ':' || c == '-'))
        .filter(|s| !s.is_empty())
        .collect()
}

const MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december",
];

/// Full month name or its 3-letter form ("oct", "october", "sept").
fn month(tok: &str) -> Option<u32> {
    MONTHS
        .iter()
        .position(|m| tok == *m || tok == &m[..3] || (*m == "september" && tok == "sept"))
        .map(|i| i as u32 + 1)
}

fn day_num(tok: &str) -> Option<u32> {
    let digits = tok.trim_end_matches(|c: char| c.is_alphabetic());
    let suffix = &tok[digits.len()..];
    if !matches!(suffix, "" | "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse().ok().filter(|d| (1..=31).contains(d))
}

/// Full day names plus unambiguous short forms ("sat" and "sun" are ordinary words too).
fn weekday(tok: &str) -> Option<Weekday> {
    Some(match tok {
        "monday" | "mon" => Weekday::Mon,
        "tuesday" | "tue" | "tues" => Weekday::Tue,
        "wednesday" => Weekday::Wed,
        "thursday" | "thu" | "thur" | "thurs" => Weekday::Thu,
        "friday" | "fri" => Weekday::Fri,
        "saturday" => Weekday::Sat,
        "sunday" => Weekday::Sun,
        _ => return None,
    })
}

/// "5pm", "5:30pm", "17:00", "5" + "pm".
fn time_at(toks: &[&str], i: usize) -> Option<NaiveTime> {
    let tok = toks[i];
    let (body, mer) = if let Some(b) = tok.strip_suffix("am") {
        (b, Some(false))
    } else if let Some(b) = tok.strip_suffix("pm") {
        (b, Some(true))
    } else {
        match toks.get(i + 1) {
            Some(&"am") => (tok, Some(false)),
            Some(&"pm") => (tok, Some(true)),
            _ => (tok, None),
        }
    };
    let (h, m) = match body.split_once(':') {
        Some((h, m)) => (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?),
        None if mer.is_some() => (body.parse::<u32>().ok()?, 0),
        None => return None, // a bare number isn't a time
    };
    let h = match mer {
        Some(pm) if (1..=12).contains(&h) => (h % 12) + if pm { 12 } else { 0 },
        Some(_) => return None,
        None if h < 24 => h,
        None => return None,
    };
    NaiveTime::from_hms_opt(h, m, 0)
}

pub fn parse_due(t: &str, now: DateTime<Local>) -> Option<i64> {
    let toks = tokens(t);
    let today = now.date_naive();
    let mut date: Option<NaiveDate> = None;
    let mut time: Option<NaiveTime> = None;

    for (i, &tok) in toks.iter().enumerate() {
        // Relative: "in 2 hours", "in 30 min"
        if tok == "in" {
            if let (Some(n), Some(unit)) = (toks.get(i + 1).and_then(|n| n.parse::<i64>().ok()), toks.get(i + 2)) {
                let d = match *unit {
                    u if u.starts_with("min") => Some(Duration::minutes(n)),
                    u if u.starts_with("hour") || u == "hr" || u == "hrs" || u == "h" => Some(Duration::hours(n)),
                    u if u.starts_with("day") => Some(Duration::days(n)),
                    _ => None,
                };
                if let Some(d) = d {
                    return Some((now + d).timestamp());
                }
            }
        }
        match tok {
            "today" | "tdy" => date = date.or(Some(today)),
            "tonight" => {
                date = date.or(Some(today));
                time = time.or(NaiveTime::from_hms_opt(21, 0, 0));
            }
            "tomorrow" | "tmrw" | "tmr" | "tomo" => date = date.or(Some(today + Duration::days(1))),
            "noon" => time = time.or(NaiveTime::from_hms_opt(12, 0, 0)),
            "midnight" => time = time.or(NaiveTime::from_hms_opt(23, 59, 0)),
            "eod" => time = time.or(NaiveTime::from_hms_opt(18, 0, 0)),
            "week" if i > 0 && toks[i - 1] == "next" => date = date.or(Some(today + Duration::days(7))),
            _ => {}
        }
        if tok == "end" && toks.get(i + 1) == Some(&"of") && toks.get(i + 2) == Some(&"day") {
            time = time.or(NaiveTime::from_hms_opt(18, 0, 0));
        }
        if date.is_none() {
            if let Some(wd) = weekday(tok) {
                let ahead = (wd.num_days_from_monday() as i64 - today.weekday().num_days_from_monday() as i64).rem_euclid(7);
                let ahead = if ahead == 0 || (i > 0 && toks[i - 1] == "next") { ahead + 7 } else { ahead };
                date = Some(today + Duration::days(ahead));
            }
        }
        if date.is_none() {
            // "oct 10", "10 oct", "10th of october"
            let m_here = month(tok);
            let pair = m_here
                .and_then(|m| toks.get(i + 1).and_then(|d| day_num(d)).map(|d| (m, d)))
                .or_else(|| {
                    day_num(tok).and_then(|d| {
                        let next = if toks.get(i + 1) == Some(&"of") { i + 2 } else { i + 1 };
                        toks.get(next).and_then(|m| month(m)).map(|m| (m, d))
                    })
                });
            if let Some((m, d)) = pair {
                let mut y = today.year();
                if let Some(nd) = NaiveDate::from_ymd_opt(y, m, d) {
                    if nd < today - Duration::days(1) {
                        y += 1;
                    }
                }
                date = NaiveDate::from_ymd_opt(y, m, d);
            }
        }
        if date.is_none() && tok.len() == 10 {
            date = NaiveDate::parse_from_str(tok, "%Y-%m-%d").ok();
        }
        if time.is_none() {
            time = time_at(&toks, i);
        }
    }

    let (date, time) = match (date, time) {
        (None, None) => return None,
        // A time alone means its next occurrence.
        (None, Some(t)) => (if t > now.time() { today } else { today + Duration::days(1) }, t),
        (Some(d), t) => (d, t.unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 0).unwrap())),
    };
    Local.from_local_datetime(&date.and_time(time)).earliest().map(|t| t.timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sunday 4 Oct 2026, 15:00 local.
    fn now() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 10, 4, 15, 0, 0).unwrap()
    }
    fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> i64 {
        Local.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap().timestamp()
    }

    #[test]
    fn strong_keyword_without_date() {
        let f = detect("Reminder: pay the electricity bill", now()).unwrap();
        assert_eq!(f.due, None);
        assert_eq!(f.reason, "reminder");
    }

    #[test]
    fn assignment_due_friday_5pm() {
        let f = detect("Assignment 3 is due Friday at 5pm", now()).unwrap();
        assert_eq!(f.due, Some(at(2026, 10, 9, 17, 0)));
    }

    #[test]
    fn meeting_tomorrow_morning() {
        let f = detect("Team meeting tomorrow 10:30 am", now()).unwrap();
        assert_eq!(f.due, Some(at(2026, 10, 5, 10, 30)));
    }

    #[test]
    fn month_day_dates() {
        assert_eq!(parse_due("submit by oct 10", now()), Some(at(2026, 10, 10, 23, 59)));
        assert_eq!(parse_due("renewal on 3rd of january", now()), Some(at(2027, 1, 3, 23, 59)));
    }

    #[test]
    fn relative_and_tonight() {
        assert_eq!(parse_due("call me in 2 hours", now()), Some(at(2026, 10, 4, 17, 0)));
        assert_eq!(parse_due("finish it tonight", now()), Some(at(2026, 10, 4, 21, 0)));
    }

    #[test]
    fn time_alone_is_next_occurrence() {
        assert_eq!(parse_due("standup at 9am", now()), Some(at(2026, 10, 5, 9, 0)));
        assert_eq!(parse_due("dinner at 19:30", now()), Some(at(2026, 10, 4, 19, 30)));
    }

    #[test]
    fn chatter_is_not_a_task() {
        assert_eq!(detect("lol see you", now()), None);
        assert_eq!(detect("I may be late", now()), None);
        assert_eq!(detect("the build has 5 warnings", now()), None);
        assert_eq!(detect("dueling banjos", now()), None);
    }

    #[test]
    fn weak_word_needs_a_date() {
        assert!(detect("can you review this", now()).is_none());
        assert!(detect("can you review this by tomorrow", now()).is_some());
    }
}
