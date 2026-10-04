//! Optional LLM, bring-your-own-key. Talks the OpenAI chat-completions format,
//! which Groq, Gemini, OpenRouter, Ollama and most others accept.

use std::time::Duration;

use serde_json::{json, Value};

use crate::{secrets, settings::Ai};

/// (base url, default model) for the built-in providers.
fn preset(provider: &str) -> Option<(&'static str, &'static str)> {
    match provider {
        "groq" => Some(("https://api.groq.com/openai/v1", "llama-3.3-70b-versatile")),
        "gemini" => Some(("https://generativelanguage.googleapis.com/v1beta/openai", "gemini-2.5-flash")),
        _ => None,
    }
}

pub fn enabled(ai: &Ai) -> bool {
    ai.provider != "off" && !ai.provider.is_empty()
}

pub fn chat(ai: &Ai, system: &str, user: &str) -> Result<String, String> {
    if !enabled(ai) {
        return Err("AI is off".into());
    }
    let (base, model) = match preset(&ai.provider) {
        Some((b, m)) => (
            if ai.base_url.is_empty() { b.to_string() } else { ai.base_url.clone() },
            if ai.model.is_empty() { m.to_string() } else { ai.model.clone() },
        ),
        None => (ai.base_url.clone(), ai.model.clone()),
    };
    if base.is_empty() || model.is_empty() {
        return Err("Set a base URL and model".into());
    }
    // Local servers like Ollama don't need a key.
    let key = secrets::get("ai").unwrap_or_default();

    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut req = agent.post(format!("{}/chat/completions", base.trim_end_matches('/')));
    if !key.is_empty() {
        req = req.header("Authorization", format!("Bearer {key}"));
    }
    let mut resp = req
        .send_json(json!({
            "model": model,
            "temperature": 0,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        }))
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let body: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    if !status.is_success() {
        let msg = body["error"]["message"].as_str().or(body[0]["error"]["message"].as_str()).unwrap_or("request failed");
        return Err(format!("{status}: {msg}"));
    }
    body["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "empty reply".into())
}

pub struct Extracted {
    pub title: String,
    pub due: Option<i64>,
}

/// Ask the model whether `text` is something the user has to do, and by when.
/// `Ok(None)` means "not a task".
pub fn extract_task(ai: &Ai, text: &str, now: chrono::DateTime<chrono::Local>) -> Result<Option<Extracted>, String> {
    let system = format!(
        "You read short messages and notifications and decide if they ask the reader to do something. \
         The current local time is {}. Reply with JSON only: \
         {{\"task\": true|false, \"title\": \"short imperative task title\", \"due\": \"YYYY-MM-DDTHH:MM\" or null}}. \
         Chit-chat, promotions and status updates are not tasks.",
        now.format("%A %Y-%m-%d %H:%M")
    );
    let reply = chat(ai, &system, text)?;
    let json = reply.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
    let v: Value = serde_json::from_str(json).map_err(|e| format!("bad reply: {e}"))?;
    if !v["task"].as_bool().unwrap_or(false) {
        return Ok(None);
    }
    let due = v["due"]
        .as_str()
        .and_then(|d| chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%dT%H:%M").ok())
        .and_then(|d| d.and_local_timezone(chrono::Local).earliest())
        .map(|d| d.timestamp());
    Ok(Some(Extracted { title: v["title"].as_str().unwrap_or(text).to_string(), due }))
}
