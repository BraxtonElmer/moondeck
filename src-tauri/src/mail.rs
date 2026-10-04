//! Email as a task source, read-only.
//! - Gmail over IMAP with an app password.
//! - Outlook / Microsoft 365 over Microsoft Graph, signed in with the device-code flow
//!   against the user's own Azure app registration (Mail.Read, public client).

use std::{path::PathBuf, sync::Arc, time::Duration};

use base64::Engine;
use chrono::{DateTime, Local, Utc};
use serde_json::Value;

use crate::{
    imap::Imap,
    inbox, secrets,
    settings::{self, Settings},
    store::{NewTask, Signal, Store},
};

const POLL: Duration = Duration::from_secs(5 * 60);
/// On first connect, only look this far back.
const FIRST_LOOKBACK_DAYS: i64 = 2;
const MAX_PER_POLL: usize = 40;

pub fn spawn(store: Arc<Store>, dir: PathBuf) {
    std::thread::Builder::new()
        .name("mail".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(20));
            let _ = check(&store, &dir);
            std::thread::sleep(POLL);
        })
        .expect("failed to start mail thread");
}

/// Poll every enabled account now. Returns a short summary for the UI.
pub fn check(store: &Store, dir: &std::path::Path) -> Result<String, String> {
    let s = settings::load(dir);
    let mut parts = Vec::new();
    if s.gmail.enabled {
        match gmail(store, &s) {
            Ok(n) => parts.push(format!("Gmail: {n} new")),
            Err(e) => {
                eprintln!("gmail: {e}");
                parts.push(format!("Gmail: {e}"));
            }
        }
    }
    if s.outlook.enabled {
        match outlook(store, &s) {
            Ok(n) => parts.push(format!("Outlook: {n} new")),
            Err(e) => {
                eprintln!("outlook: {e}");
                parts.push(format!("Outlook: {e}"));
            }
        }
    }
    if parts.is_empty() {
        return Err("No email accounts are switched on".into());
    }
    Ok(parts.join(" · "))
}

/// Store the message as a signal and, if it reads like a task, as an inbox task.
fn ingest(store: &Store, s: &Settings, source: &str, id: &str, from: &str, subject: &str, snippet: &str, received: i64) -> bool {
    let app = if source == "gmail" { "Gmail" } else { "Outlook" };
    let _ = store.insert(&Signal {
        start: received,
        end: None,
        source: "email".into(),
        kind: source.into(),
        subject: from.into(),
        label: app.into(),
        detail: subject.into(),
    });
    let text = format!("{subject}. {snippet}");
    let Some(c) = inbox::classify(&text, subject, s) else { return false };
    store
        .add_task(&NewTask {
            created: received,
            source: source.into(),
            reference: id.into(),
            app: app.into(),
            title: c.title,
            detail: format!("{} · {app}", display_name(from)),
            due: c.due,
        })
        .unwrap_or(false)
}

// ---------- Gmail ----------

fn gmail(store: &Store, s: &Settings) -> Result<usize, String> {
    let pass = secrets::get("gmail").ok_or("no app password saved")?;
    let mut imap = Imap::connect("imap.gmail.com")?;
    imap.login(&s.gmail.address, &pass)?;
    imap.examine("INBOX")?;

    let last: u32 = store.get_state("gmail.uid").and_then(|v| v.parse().ok()).unwrap_or(0);
    let since = (Local::now() - chrono::Duration::days(FIRST_LOOKBACK_DAYS)).format("%-d-%b-%Y").to_string();
    let mut uids: Vec<u32> = imap.uids_since(&since)?.into_iter().filter(|u| *u > last).collect();
    uids.sort_unstable();
    let uids = &uids[uids.len().saturating_sub(MAX_PER_POLL)..];

    let mut found = 0;
    let mut max = last;
    for m in imap.fetch(uids)? {
        max = max.max(m.uid);
        let h = headers(&m.header);
        let received = h.date.unwrap_or_else(|| Utc::now().timestamp());
        if ingest(store, s, "gmail", &format!("gmail:{}", m.uid), &h.from, &h.subject, &snippet(&m.body), received) {
            found += 1;
        }
    }
    imap.logout();
    store.set_state("gmail.uid", &max.to_string()).map_err(|e| e.to_string())?;
    Ok(found)
}

struct Headers {
    from: String,
    subject: String,
    date: Option<i64>,
}

fn headers(raw: &str) -> Headers {
    // Unfold continuation lines first.
    let unfolded = raw.replace("\r\n ", " ").replace("\r\n\t", " ").replace("\n ", " ").replace("\n\t", " ");
    let mut h = Headers { from: String::new(), subject: String::new(), date: None };
    for line in unfolded.lines() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim();
        match k.to_ascii_lowercase().as_str() {
            "from" => h.from = decode_words(v),
            "subject" => h.subject = decode_words(v),
            "date" => h.date = DateTime::parse_from_rfc2822(v.split(" (").next().unwrap_or(v)).ok().map(|d| d.timestamp()),
            _ => {}
        }
    }
    h
}

/// "Priya Shah <priya@x.com>" -> "Priya Shah"
fn display_name(from: &str) -> String {
    let name = from.split('<').next().unwrap_or(from).trim().trim_matches('"');
    if name.is_empty() { from.trim_matches(['<', '>']).to_string() } else { name.to_string() }
}

/// RFC 2047 encoded words: =?UTF-8?B?...?= and =?UTF-8?Q?...?=
fn decode_words(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    let mut last_was_word = false;
    while let Some(start) = rest.find("=?") {
        let before = &rest[..start];
        // Whitespace between two encoded words is dropped.
        if !(last_was_word && before.trim().is_empty()) {
            out.push_str(before);
        }
        let tail = &rest[start + 2..];
        let parts: Vec<&str> = tail.splitn(3, '?').collect();
        let decoded = (parts.len() == 3).then(|| {
            let end = parts[2].find("?=")?;
            let data = &parts[2][..end];
            let bytes = match parts[1].to_ascii_uppercase().as_str() {
                "B" => base64::engine::general_purpose::STANDARD.decode(data).ok()?,
                "Q" => qp(&data.replace('_', " ")),
                _ => return None,
            };
            let consumed = 2 + parts[0].len() + 1 + parts[1].len() + 1 + end + 2;
            Some((String::from_utf8_lossy(&bytes).into_owned(), consumed))
        });
        match decoded.flatten() {
            Some((text, consumed)) => {
                out.push_str(&text);
                rest = &rest[start + consumed..];
                last_was_word = true;
            }
            None => {
                out.push_str("=?");
                rest = &rest[start + 2..];
                last_was_word = false;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Quoted-printable decode.
fn qp(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'=' {
            if b.get(i + 1) == Some(&b'\r') && b.get(i + 2) == Some(&b'\n') {
                i += 3;
                continue;
            }
            if b.get(i + 1) == Some(&b'\n') {
                i += 2;
                continue;
            }
            if let Some(v) = b.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(std::str::from_utf8(h).ok()?, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// Best-effort plain text from the start of a body part: base64 or quoted-printable, HTML stripped.
fn snippet(body: &[u8]) -> String {
    let raw = String::from_utf8_lossy(body);
    let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let looks_b64 = compact.len() > 40
        && compact.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
    let text = if looks_b64 {
        // The snippet may be cut mid-block; decode whole 4-char groups only.
        let usable = &compact[..compact.len() / 4 * 4];
        base64::engine::general_purpose::STANDARD
            .decode(usable)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    } else if raw.contains("=\r\n") || raw.contains("=3D") || raw.contains("=20") {
        String::from_utf8_lossy(&qp(&raw)).into_owned()
    } else {
        raw.into_owned()
    };
    let mut plain = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                plain.push(' ');
            }
            _ if !in_tag => plain.push(c),
            _ => {}
        }
    }
    inbox::shorten(&plain, 600)
}

// ---------- Outlook ----------

const AUTH: &str = "https://login.microsoftonline.com/common/oauth2/v2.0";
const SCOPE: &str = "offline_access Mail.Read User.Read";

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .build()
        .new_agent()
}

fn post_form(url: &str, form: &[(&str, &str)]) -> Result<Value, String> {
    let mut resp = agent().post(url).send_form(form.iter().copied()).map_err(|e| e.to_string())?;
    resp.body_mut().read_json().map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct DeviceCode {
    pub user_code: String,
    pub verification_uri: String,
    #[serde(skip)]
    device_code: String,
    #[serde(skip)]
    interval: u64,
    #[serde(skip)]
    expires_in: u64,
}

/// Step 1 of sign-in: get a code for the user to enter at microsoft.com/devicelogin.
pub fn outlook_start(client_id: &str) -> Result<DeviceCode, String> {
    if client_id.trim().is_empty() {
        return Err("Add your Application (client) ID first".into());
    }
    let v = post_form(&format!("{AUTH}/devicecode"), &[("client_id", client_id), ("scope", SCOPE)])?;
    if let Some(e) = v["error_description"].as_str() {
        return Err(e.lines().next().unwrap_or(e).to_string());
    }
    Ok(DeviceCode {
        user_code: v["user_code"].as_str().unwrap_or_default().into(),
        verification_uri: v["verification_uri"].as_str().unwrap_or("https://microsoft.com/devicelogin").into(),
        device_code: v["device_code"].as_str().unwrap_or_default().into(),
        interval: v["interval"].as_u64().unwrap_or(5),
        expires_in: v["expires_in"].as_u64().unwrap_or(900),
    })
}

/// Step 2: wait for the user to finish signing in, then keep the refresh token.
/// Returns the signed-in mailbox address.
pub fn outlook_finish(client_id: &str, code: &DeviceCode) -> Result<String, String> {
    let deadline = std::time::Instant::now() + Duration::from_secs(code.expires_in);
    let mut interval = code.interval.max(2);
    while std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_secs(interval));
        let v = post_form(
            &format!("{AUTH}/token"),
            &[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id),
                ("device_code", &code.device_code),
            ],
        )?;
        match v["error"].as_str() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                interval += 5;
                continue;
            }
            Some(_) => return Err(v["error_description"].as_str().unwrap_or("sign-in failed").lines().next().unwrap_or("").into()),
            None => {}
        }
        let refresh = v["refresh_token"].as_str().ok_or("no refresh token returned")?;
        secrets::set("outlook", refresh)?;
        let access = v["access_token"].as_str().ok_or("no access token returned")?;
        let me = graph_get(access, "https://graph.microsoft.com/v1.0/me?$select=mail,userPrincipalName")?;
        return Ok(me["mail"].as_str().or(me["userPrincipalName"].as_str()).unwrap_or("signed in").into());
    }
    Err("Sign-in timed out".into())
}

fn graph_get(token: &str, url: &str) -> Result<Value, String> {
    let mut resp = agent()
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    let v: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(v["error"]["message"].as_str().unwrap_or("Graph request failed").into());
    }
    Ok(v)
}

fn outlook(store: &Store, s: &Settings) -> Result<usize, String> {
    let refresh = secrets::get("outlook").ok_or("not signed in")?;
    let v = post_form(
        &format!("{AUTH}/token"),
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &s.outlook.client_id),
            ("refresh_token", &refresh),
            ("scope", SCOPE),
        ],
    )?;
    let access = v["access_token"].as_str().ok_or_else(|| {
        v["error_description"].as_str().unwrap_or("token refresh failed").lines().next().unwrap_or("").to_string()
    })?;
    if let Some(new_refresh) = v["refresh_token"].as_str() {
        secrets::set("outlook", new_refresh)?;
    }

    let since = store
        .get_state("outlook.since")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or_else(|| (Utc::now() - chrono::Duration::days(FIRST_LOOKBACK_DAYS)).timestamp());
    let since_iso = DateTime::from_timestamp(since, 0).unwrap_or_default().format("%Y-%m-%dT%H:%M:%SZ");
    let url = format!(
        "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages?$select=id,subject,from,receivedDateTime,bodyPreview&$orderby=receivedDateTime%20desc&$top={MAX_PER_POLL}&$filter=receivedDateTime%20gt%20{since_iso}"
    );
    let v = graph_get(access, &url)?;

    let mut found = 0;
    let mut newest = since;
    for m in v["value"].as_array().cloned().unwrap_or_default() {
        let received = m["receivedDateTime"]
            .as_str()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.timestamp())
            .unwrap_or(since);
        newest = newest.max(received);
        let from = format!(
            "{} <{}>",
            m["from"]["emailAddress"]["name"].as_str().unwrap_or(""),
            m["from"]["emailAddress"]["address"].as_str().unwrap_or("")
        );
        let id = format!("outlook:{}", m["id"].as_str().unwrap_or_default());
        if ingest(store, s, "outlook", &id, &from, m["subject"].as_str().unwrap_or(""), m["bodyPreview"].as_str().unwrap_or(""), received) {
            found += 1;
        }
    }
    store.set_state("outlook.since", &newest.to_string()).map_err(|e| e.to_string())?;
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_subjects() {
        assert_eq!(decode_words("=?UTF-8?B?QXNzaWdubWVudCBkdWU=?= Friday"), "Assignment due Friday");
        assert_eq!(decode_words("=?utf-8?Q?Invoice_=E2=82=AC40?="), "Invoice €40");
        assert_eq!(decode_words("=?UTF-8?Q?a?= =?UTF-8?Q?b?="), "ab");
        assert_eq!(decode_words("plain subject"), "plain subject");
    }

    #[test]
    fn parses_headers() {
        let h = headers("From: \"Priya Shah\" <priya@x.com>\r\nSubject: Slides\r\n by Friday\r\nDate: Sun, 4 Oct 2026 10:00:00 +0000\r\n");
        assert_eq!(display_name(&h.from), "Priya Shah");
        assert_eq!(h.subject, "Slides by Friday");
        assert_eq!(h.date, Some(1_791_108_000));
    }

    #[test]
    fn body_snippets() {
        assert_eq!(snippet(b"Please send it by =\r\n5pm =3D thanks"), "Please send it by 5pm = thanks");
        assert_eq!(snippet(b"<p>Hi <b>there</b></p>"), "Hi there");
        let b64 = base64::engine::general_purpose::STANDARD.encode("Your invoice is due tomorrow, please pay soon.");
        assert_eq!(snippet(b64.as_bytes()), "Your invoice is due tomorrow, please pay soon.");
    }
}
