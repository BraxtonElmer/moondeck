//! Genshin Impact via HoYoLAB Real-Time Notes. Read-only: commissions, resin, expeditions.
//!
//! Needs the user's HoYoLAB cookie (`ltoken_v2=...; ltuid_v2=...`) in Credential Manager
//! and Real-Time Notes switched on in their Battle Chronicle.

use std::{path::PathBuf, sync::Arc, time::Duration};

use chrono::Local;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    secrets, settings,
    store::{NewTask, Signal, Store},
};

const POLL: Duration = Duration::from_secs(20 * 60);
/// HoYoLAB's overseas "dynamic secret" salt for the web client.
const DS_SALT: &str = "6s25p5ox5y14umn1p61aqyyvbvvl3lrt";
/// Raise a "resin almost full" task once this close to the cap.
const RESIN_MARGIN: u32 = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Notes {
    pub commissions_done: u32,
    pub commissions_total: u32,
    /// The extra reward from Katheryne after all four.
    pub bonus_claimed: bool,
    pub resin: u32,
    pub resin_max: u32,
    /// Seconds until resin is full.
    pub resin_full_in: u64,
    pub expeditions_done: u32,
    pub expeditions_total: u32,
}

impl Notes {
    pub fn summary(&self) -> String {
        format!(
            "{}/{} commissions{} · resin {}/{} · expeditions {}/{}",
            self.commissions_done,
            self.commissions_total,
            if self.bonus_claimed { "" } else { " (bonus unclaimed)" },
            self.resin,
            self.resin_max,
            self.expeditions_done,
            self.expeditions_total
        )
    }
}

pub fn spawn(store: Arc<Store>, dir: PathBuf) {
    std::thread::Builder::new()
        .name("hoyolab".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_secs(30));
            if settings::load(&dir).hoyolab.enabled {
                if let Err(e) = check(&store) {
                    eprintln!("hoyolab: {e}");
                }
            }
            std::thread::sleep(POLL);
        })
        .expect("failed to start hoyolab thread");
}

/// Fetch notes now, record them, and raise resin/expedition tasks. Returns a summary.
pub fn check(store: &Store) -> Result<String, String> {
    let cookie = secrets::get("hoyolab").ok_or("Paste your HoYoLAB cookie first")?;
    let notes = fetch(&cookie)?;
    let now = chrono::Utc::now().timestamp();
    let _ = store.insert(&Signal {
        start: now,
        end: None,
        source: "hoyolab".into(),
        kind: "notes".into(),
        subject: "genshin".into(),
        label: "Genshin Impact".into(),
        detail: serde_json::to_string(&notes).unwrap_or_default(),
    });

    let day = Local::now().format("%Y-%m-%d");
    if notes.resin_max > 0 && notes.resin + RESIN_MARGIN >= notes.resin_max {
        let _ = store.add_task(&NewTask {
            created: now,
            source: "hoyolab".into(),
            reference: format!("resin:{day}"),
            app: "Genshin".into(),
            title: format!("Spend resin ({}/{})", notes.resin, notes.resin_max),
            detail: "Resin is about to cap · Genshin".into(),
            due: Some(now + notes.resin_full_in as i64),
        });
    }
    if notes.expeditions_total > 0 && notes.expeditions_done == notes.expeditions_total {
        let _ = store.add_task(&NewTask {
            created: now,
            source: "hoyolab".into(),
            reference: format!("expeditions:{day}"),
            app: "Genshin".into(),
            title: "Collect expeditions".into(),
            detail: format!("{} expeditions finished · Genshin", notes.expeditions_done),
            due: None,
        });
    }
    Ok(notes.summary())
}

/// Most recent notes recorded between `from` and `to`.
pub fn latest(store: &Store, from: i64, to: i64) -> Option<Notes> {
    store
        .between("hoyolab", from, to)
        .ok()?
        .into_iter()
        .rev()
        .find_map(|s| serde_json::from_str(&s.detail).ok())
}

// ---------- API ----------

fn ds() -> String {
    let t = chrono::Utc::now().timestamp();
    let seed = chrono::Utc::now().timestamp_subsec_nanos() as u64 ^ (t as u64).rotate_left(17);
    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let r: String = (0..6).map(|i| chars[((seed >> (i * 5)) % chars.len() as u64) as usize] as char).collect();
    let hash = Md5::digest(format!("salt={DS_SALT}&t={t}&r={r}").as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("{t},{r},{hex}")
}

fn get(url: &str, cookie: &str) -> Result<Value, String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .http_status_as_error(false)
        .build()
        .new_agent();
    let mut resp = agent
        .get(url)
        .header("Cookie", cookie)
        .header("DS", ds())
        .header("x-rpc-app_version", "1.5.0")
        .header("x-rpc-client_type", "5")
        .header("x-rpc-language", "en-us")
        .header("Referer", "https://act.hoyolab.com/")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36")
        .call()
        .map_err(|e| e.to_string())?;
    let v: Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    match v["retcode"].as_i64() {
        Some(0) => Ok(v["data"].clone()),
        Some(10102) => Err("Turn on Real-Time Notes in your HoYoLAB Battle Chronicle".into()),
        Some(-100) | Some(10001) => Err("HoYoLAB cookie expired or invalid".into()),
        _ => Err(v["message"].as_str().unwrap_or("HoYoLAB request failed").to_string()),
    }
}

fn fetch(cookie: &str) -> Result<Notes, String> {
    let roles = get(
        "https://api-account-os.hoyolab.com/account/binding/api/getUserGameRolesByCookie?game_biz=hk4e_global",
        cookie,
    )?;
    let role = roles["list"]
        .as_array()
        .and_then(|l| l.iter().max_by_key(|r| r["level"].as_i64().unwrap_or(0)))
        .ok_or("No Genshin account on this HoYoLAB profile")?;
    let uid = role["game_uid"].as_str().ok_or("missing uid")?;
    let region = role["region"].as_str().ok_or("missing region")?;
    let data = get(
        &format!("https://bbs-api-os.hoyolab.com/game_record/genshin/api/dailyNote?server={region}&role_id={uid}"),
        cookie,
    )?;
    Ok(parse(&data))
}

fn num(v: &Value) -> u32 {
    v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())).unwrap_or(0) as u32
}

fn parse(d: &Value) -> Notes {
    // Newer responses nest commissions under `daily_task`.
    let task = if d["daily_task"].is_object() { &d["daily_task"] } else { d };
    let done_field = if task["finished_num"].is_null() { "finished_task_num" } else { "finished_num" };
    let total_field = if task["total_num"].is_null() { "total_task_num" } else { "total_num" };
    let expeditions = d["expeditions"].as_array().cloned().unwrap_or_default();
    Notes {
        commissions_done: num(&task[done_field]),
        commissions_total: num(&task[total_field]),
        bonus_claimed: task["is_extra_task_reward_received"].as_bool().unwrap_or(false),
        resin: num(&d["current_resin"]),
        resin_max: num(&d["max_resin"]),
        resin_full_in: num(&d["resin_recovery_time"]) as u64,
        expeditions_done: expeditions.iter().filter(|e| e["status"] == "Finished").count() as u32,
        expeditions_total: num(&d["max_expedition_num"]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_old_and_new_shapes() {
        let old = serde_json::json!({
            "current_resin": 142, "max_resin": 200, "resin_recovery_time": "27840",
            "finished_task_num": 3, "total_task_num": 4, "is_extra_task_reward_received": false,
            "max_expedition_num": 5,
            "expeditions": [{ "status": "Finished" }, { "status": "Ongoing" }, { "status": "Finished" }]
        });
        let n = parse(&old);
        assert_eq!((n.commissions_done, n.commissions_total, n.resin, n.resin_full_in), (3, 4, 142, 27840));
        assert_eq!((n.expeditions_done, n.expeditions_total), (2, 5));

        let new = serde_json::json!({
            "current_resin": 200, "max_resin": 200, "resin_recovery_time": "0",
            "daily_task": { "finished_num": 4, "total_num": 4, "is_extra_task_reward_received": true },
            "max_expedition_num": 5, "expeditions": []
        });
        let n = parse(&new);
        assert_eq!((n.commissions_done, n.bonus_claimed), (4, true));
    }

    #[test]
    fn ds_shape() {
        let ds = ds();
        let parts: Vec<&str> = ds.split(',').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1].len(), 6);
        assert_eq!(parts[2].len(), 32);
    }
}
