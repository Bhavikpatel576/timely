use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Json, Response};

use crate::db;
use crate::db::config_store;

pub async fn require_api_key(req: Request, next: Next) -> Response {
    // Check if hub has an API key configured
    let stored_key = tokio::task::spawn_blocking(|| {
        let conn = match db::open_default_db() {
            Ok(c) => c,
            Err(_) => return None,
        };
        config_store::get(&conn, "sync.api_key").ok().flatten()
    })
    .await
    .unwrap_or(None);

    // No key configured on hub → open mode, allow all requests
    if stored_key.is_none() {
        return next.run(req).await;
    }
    let stored_key = stored_key.unwrap();

    // Key is configured → require matching X-API-Key header
    let client_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match client_key {
        Some(k) if k == stored_key => next.run(req).await,
        Some(_) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "ok": false, "error": "Invalid API key", "error_code": "unauthorized" })),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "ok": false, "error": "Missing X-API-Key header (hub has auth enabled)", "error_code": "unauthorized" })),
        )
            .into_response(),
    }
}
