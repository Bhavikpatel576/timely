use rusqlite::Connection;
use serde::Serialize;

use crate::db::config_store;
use crate::db::events;
use crate::db::sync as db_sync;
use crate::error::{Result, TimelyError};
use crate::types::Device;

const BATCH_SIZE: i64 = 1000;

#[derive(Debug, Serialize)]
struct PushRequestBody {
    device: PushDeviceBody,
    events: Vec<PushEventBody>,
}

#[derive(Debug, Serialize)]
struct PushDeviceBody {
    id: String,
    name: String,
    platform: String,
}

#[derive(Debug, Serialize)]
struct PushEventBody {
    timestamp: String,
    duration: f64,
    app: String,
    title: String,
    url: Option<String>,
    url_domain: Option<String>,
    category_name: Option<String>,
    is_afk: bool,
}

#[derive(Debug)]
pub struct SyncPushResult {
    pub total_accepted: usize,
    pub total_duplicates: usize,
    pub batches: usize,
}

/// Build a request with optional API key header.
fn add_auth(builder: reqwest::blocking::RequestBuilder, api_key: &Option<String>) -> reqwest::blocking::RequestBuilder {
    match api_key {
        Some(key) => builder.header("X-API-Key", key),
        None => builder,
    }
}

pub fn push_events(conn: &Connection, device: &Device) -> Result<SyncPushResult> {
    let hub_url = config_store::get(conn, "sync.hub_url")?
        .ok_or_else(|| TimelyError::Config("sync.hub_url not configured".into()))?;
    let api_key = config_store::get(conn, "sync.api_key")?;

    let last_synced_id = db_sync::get_sync_log(conn, &device.id)?
        .map(|(id, _)| id)
        .unwrap_or(0);

    let client = reqwest::blocking::Client::new();
    let push_url = format!("{}/api/sync/push", hub_url.trim_end_matches('/'));

    let mut cursor = last_synced_id;
    let mut total_accepted = 0usize;
    let mut total_duplicates = 0usize;
    let mut batches = 0usize;

    loop {
        let batch = events::query_events_after_id(conn, &device.id, cursor, BATCH_SIZE)?;
        if batch.is_empty() {
            break;
        }

        let last_id = batch.last().map(|e| e.id).unwrap_or(cursor);

        let push_events: Vec<PushEventBody> = batch
            .iter()
            .map(|e| PushEventBody {
                timestamp: e.timestamp.to_rfc3339(),
                duration: e.duration,
                app: e.app.clone(),
                title: e.title.clone(),
                url: e.url.clone(),
                url_domain: e.url_domain.clone(),
                category_name: e.category_name.clone(),
                is_afk: e.is_afk,
            })
            .collect();

        let body = PushRequestBody {
            device: PushDeviceBody {
                id: device.id.clone(),
                name: device.name.clone(),
                platform: device.platform.clone(),
            },
            events: push_events,
        };

        let resp = add_auth(client.post(&push_url), &api_key)
            .json(&body)
            .send()
            .map_err(|e| TimelyError::Generic(format!("sync push failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(TimelyError::Generic(format!(
                "sync push returned {}: {}",
                status, text
            )));
        }

        let result: serde_json::Value = resp
            .json()
            .map_err(|e| TimelyError::Generic(format!("sync response parse error: {}", e)))?;

        if let Some(data) = result.get("data") {
            total_accepted += data["accepted"].as_u64().unwrap_or(0) as usize;
            total_duplicates += data["duplicates"].as_u64().unwrap_or(0) as usize;
        }

        // Update sync log after each successful batch
        db_sync::update_sync_log(conn, &device.id, last_id)?;
        cursor = last_id;
        batches += 1;

        if batch.len() < BATCH_SIZE as usize {
            break;
        }
    }

    Ok(SyncPushResult {
        total_accepted,
        total_duplicates,
        batches,
    })
}

pub fn register_with_hub(conn: &Connection, device: &Device) -> Result<()> {
    let hub_url = config_store::get(conn, "sync.hub_url")?
        .ok_or_else(|| TimelyError::Config("sync.hub_url not configured".into()))?;
    let api_key = config_store::get(conn, "sync.api_key")?;

    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/sync/register", hub_url.trim_end_matches('/'));

    let resp = add_auth(client.post(&url), &api_key)
        .json(&serde_json::json!({
            "device_id": device.id,
            "name": device.name,
            "platform": device.platform,
        }))
        .send()
        .map_err(|e| TimelyError::Generic(format!("registration failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(TimelyError::Generic(format!(
            "registration returned {}: {}",
            status, text
        )));
    }

    Ok(())
}

pub fn get_hub_status(hub_url: &str, api_key: &Option<String>) -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/api/sync/status", hub_url.trim_end_matches('/'));

    let resp = add_auth(client.get(&url), api_key)
        .send()
        .map_err(|e| TimelyError::Generic(format!("status request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(TimelyError::Generic(format!(
            "status returned {}: {}",
            status, text
        )));
    }

    let result: serde_json::Value = resp
        .json()
        .map_err(|e| TimelyError::Generic(format!("status parse error: {}", e)))?;
    Ok(result)
}

pub fn fetch_remote_summary(
    hub_url: &str,
    api_key: &Option<String>,
    from: &str,
    to: &str,
    group_by: &str,
    device: Option<&str>,
) -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let mut url = format!(
        "{}/api/summary?from={}&to={}&groupBy={}",
        hub_url.trim_end_matches('/'),
        from,
        to,
        group_by,
    );
    if let Some(dev) = device {
        url.push_str(&format!("&device={}", dev));
    }

    let resp = add_auth(client.get(&url), api_key)
        .send()
        .map_err(|e| TimelyError::Generic(format!("remote summary failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(TimelyError::Generic(format!(
            "remote summary returned {}: {}",
            status, text
        )));
    }

    resp.json()
        .map_err(|e| TimelyError::Generic(format!("remote summary parse error: {}", e)))
}

pub fn fetch_remote_timeline(
    hub_url: &str,
    api_key: &Option<String>,
    from: &str,
    to: &str,
    limit: Option<i64>,
    device: Option<&str>,
) -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let mut url = format!(
        "{}/api/timeline?from={}&to={}",
        hub_url.trim_end_matches('/'),
        from,
        to,
    );
    if let Some(l) = limit {
        url.push_str(&format!("&limit={}", l));
    }
    if let Some(dev) = device {
        url.push_str(&format!("&device={}", dev));
    }

    let resp = add_auth(client.get(&url), api_key)
        .send()
        .map_err(|e| TimelyError::Generic(format!("remote timeline failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(TimelyError::Generic(format!(
            "remote timeline returned {}: {}",
            status, text
        )));
    }

    resp.json()
        .map_err(|e| TimelyError::Generic(format!("remote timeline parse error: {}", e)))
}

pub fn fetch_remote_now(
    hub_url: &str,
    api_key: &Option<String>,
    device: Option<&str>,
) -> Result<serde_json::Value> {
    let client = reqwest::blocking::Client::new();
    let mut url = format!(
        "{}/api/current",
        hub_url.trim_end_matches('/'),
    );
    if let Some(dev) = device {
        url.push_str(&format!("?device={}", dev));
    }

    let resp = add_auth(client.get(&url), api_key)
        .send()
        .map_err(|e| TimelyError::Generic(format!("remote now failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(TimelyError::Generic(format!(
            "remote now returned {}: {}",
            status, text
        )));
    }

    resp.json()
        .map_err(|e| TimelyError::Generic(format!("remote now parse error: {}", e)))
}
