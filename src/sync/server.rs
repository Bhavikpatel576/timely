use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::db::categories::get_category_by_name;
use crate::db::sync as db_sync;

#[derive(Debug, Deserialize)]
pub struct PushRequest {
    pub device: PushDevice,
    pub events: Vec<PushEvent>,
}

#[derive(Debug, Deserialize)]
pub struct PushDevice {
    pub id: String,
    pub name: String,
    pub platform: String,
}

#[derive(Debug, Deserialize)]
pub struct PushEvent {
    pub timestamp: String,
    pub duration: f64,
    pub app: String,
    pub title: String,
    pub url: Option<String>,
    pub url_domain: Option<String>,
    pub category_name: Option<String>,
    pub is_afk: bool,
}

#[derive(Debug, Serialize)]
pub struct PushResult {
    pub accepted: usize,
    pub duplicates: usize,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub device_id: String,
    pub name: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceStatus {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub last_sync: Option<String>,
    pub event_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub devices: Vec<DeviceStatus>,
    pub total_events: i64,
}

fn sync_error(msg: String) -> (StatusCode, axum::response::Response) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "ok": false, "error": msg, "error_code": "sync_error" })).into_response(),
    )
}

pub async fn handle_push(
    Json(body): Json<PushRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, axum::response::Response)> {
    let device = body.device;
    let events = body.events;

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| sync_error(e.to_string()))?;

        // Upsert the remote device
        db_sync::upsert_remote_device(&conn, &device.id, &device.name, &device.platform)
            .map_err(|e| sync_error(e.to_string()))?;

        let mut accepted = 0usize;
        let mut duplicates = 0usize;

        for event in &events {
            // Resolve category_name → category_id via hub's categories table
            let category_id = if let Some(ref cat_name) = event.category_name {
                get_category_by_name(&conn, cat_name)
                    .ok()
                    .flatten()
                    .map(|c| c.id)
            } else {
                None
            };

            let is_new = db_sync::upsert_remote_event(
                &conn,
                &device.id,
                &event.timestamp,
                event.duration,
                &event.app,
                &event.title,
                event.url.as_deref(),
                event.url_domain.as_deref(),
                category_id,
                event.is_afk,
            )
            .map_err(|e| sync_error(e.to_string()))?;

            if is_new {
                accepted += 1;
            } else {
                duplicates += 1;
            }
        }

        Ok(Json(serde_json::json!({
            "ok": true,
            "data": {
                "accepted": accepted,
                "duplicates": duplicates,
            }
        })))
    })
    .await
    .map_err(|e| sync_error(e.to_string()))?
}

pub async fn handle_register(
    Json(body): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, axum::response::Response)> {
    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| sync_error(e.to_string()))?;

        db_sync::upsert_remote_device(&conn, &body.device_id, &body.name, &body.platform)
            .map_err(|e| sync_error(e.to_string()))?;

        Ok(Json(serde_json::json!({
            "ok": true,
            "data": {
                "device_id": body.device_id,
                "name": body.name,
                "registered": true,
            }
        })))
    })
    .await
    .map_err(|e| sync_error(e.to_string()))?
}

pub async fn handle_status(
) -> Result<Json<serde_json::Value>, (StatusCode, axum::response::Response)> {
    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| sync_error(e.to_string()))?;

        let device_rows = db_sync::get_device_event_counts(&conn)
            .map_err(|e| sync_error(e.to_string()))?;

        let devices: Vec<DeviceStatus> = device_rows
            .into_iter()
            .map(|(id, name, platform, last_sync, event_count)| DeviceStatus {
                id,
                name,
                platform,
                last_sync,
                event_count,
            })
            .collect();

        let total_events = db_sync::get_total_event_count(&conn)
            .map_err(|e| sync_error(e.to_string()))?;

        let status = SyncStatusResponse {
            devices,
            total_events,
        };

        Ok(Json(serde_json::json!({
            "ok": true,
            "data": status,
        })))
    })
    .await
    .map_err(|e| sync_error(e.to_string()))?
}
