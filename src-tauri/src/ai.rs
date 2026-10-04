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
