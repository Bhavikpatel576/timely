use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;

use super::handlers;
use crate::sync::{auth, server as sync_server};

pub fn build_router() -> Router {
    let api = Router::new()
        .route("/api/summary", get(handlers::get_summary))
        .route("/api/current", get(handlers::get_current))
        .route("/api/categories", get(handlers::get_categories))
        .route("/api/apps", get(handlers::get_apps))
        .route("/api/timeline", get(handlers::get_timeline))
        .route("/api/productivity", get(handlers::get_productivity))
        .route("/api/trends", get(handlers::get_trends))
        .route("/api/apps/{name}/details", get(handlers::get_app_details))
        .route("/api/rules", get(handlers::get_rules))
        .route("/api/rules", post(handlers::post_rule))
        .route("/api/rules/{id}", put(handlers::put_rule))
        .route("/api/rules/{id}", delete(handlers::delete_rule));

    // Sync API routes — protected by API key middleware
    let sync_api = Router::new()
        .route("/api/sync/push", post(sync_server::handle_push))
        .route("/api/sync/register", post(sync_server::handle_register))
        .route("/api/sync/status", get(sync_server::handle_status))
        .layer(middleware::from_fn(auth::require_api_key));

    Router::new()
        .merge(api)
        .merge(sync_api)
        .fallback(handlers::serve_embedded)
        .layer(CorsLayer::permissive())
}
