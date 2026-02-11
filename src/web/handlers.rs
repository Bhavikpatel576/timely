use axum::extract::{Path, Query};
use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Json, Response};
use serde::Deserialize;

use crate::db;
use crate::db::categories;
use crate::query;
use crate::types::format_duration;
use crate::web::assets::DashboardAssets;

// --- Query parameter structs ---

#[derive(Deserialize)]
pub struct TimeRangeParams {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Deserialize)]
pub struct SummaryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "groupBy")]
    pub group_by: Option<String>,
    pub device: Option<String>,
}

#[derive(Deserialize)]
pub struct AppsParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct TimelineParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub limit: Option<i64>,
    pub device: Option<String>,
}

#[derive(Deserialize)]
pub struct TrendsParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub interval: Option<String>,
}

#[derive(Deserialize)]
pub struct RuleBody {
    pub app: Option<String>,
    pub category_id: Option<i64>,
    pub field: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRuleBody {
    pub category_id: Option<i64>,
}

// --- Helpers ---

fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Convert date strings (YYYY-MM-DD) to UTC-aware RFC 3339 boundaries.
/// Interprets dates as local time, converts to UTC for correct DB comparison.
fn date_range(from: Option<String>, to: Option<String>) -> (String, String) {
    let from = from.unwrap_or_else(today_str);
    let to = to.unwrap_or_else(today_str);

    let from_utc = chrono::NaiveDate::parse_from_str(&from, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .and_then(|dt| dt.and_local_timezone(chrono::Local).single())
        .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
        .unwrap_or_else(|| format!("{}T00:00:00+00:00", from));

    let to_utc = chrono::NaiveDate::parse_from_str(&to, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .and_then(|dt| dt.and_local_timezone(chrono::Local).single())
        .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
        .unwrap_or_else(|| format!("{}T23:59:59+00:00", to));

    (from_utc, to_utc)
}

fn internal_error(msg: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": msg })),
    )
}

/// Resolve a device filter string to a device_id, or None for "all"/absent.
fn resolve_device_id(conn: &rusqlite::Connection, device: &Option<String>) -> Option<String> {
    match device {
        None => None,
        Some(d) if d == "all" || d.is_empty() => None,
        Some(name) => conn
            .query_row(
                "SELECT id FROM devices WHERE name = ?1",
                rusqlite::params![name],
                |row| row.get::<_, String>(0),
            )
            .ok(),
    }
}

// --- Handlers ---

pub async fn get_summary(
    Query(params): Query<SummaryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (from_date, to_date) = date_range(params.from, params.to);
    let group_by = params.group_by.unwrap_or_else(|| "category".into());
    let device_filter = params.device;

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let device_id = resolve_device_id(&conn, &device_filter);
        let device_clause = if device_id.is_some() {
            " AND e.device_id = ?3"
        } else {
            ""
        };

        let (group_sql, _name_expr) = if group_by == "app" {
            (
                format!(
                    "SELECT CASE
                       WHEN e.url_domain IS NOT NULL AND e.url_domain != ''
                       THEN e.url_domain
                       ELSE COALESCE(e.app, 'Unknown')
                     END as label, SUM(e.duration) as total_seconds
                     FROM events e
                     WHERE e.timestamp >= ?1 AND e.timestamp <= ?2 AND e.is_afk = 0 AND e.duration > 0{}
                     GROUP BY label
                     ORDER BY total_seconds DESC",
                    device_clause,
                ),
                true,
            )
        } else {
            (
                format!(
                    "SELECT COALESCE(c.name, 'uncategorized') as name, SUM(e.duration) as total_seconds
                     FROM events e
                     LEFT JOIN categories c ON e.category_id = c.id
                     WHERE e.timestamp >= ?1 AND e.timestamp <= ?2 AND e.is_afk = 0 AND e.duration > 0{}
                     GROUP BY COALESCE(c.name, 'uncategorized')
                     ORDER BY total_seconds DESC",
                    device_clause,
                ),
                false,
            )
        };

        let mut stmt = conn.prepare(&group_sql).map_err(|e| internal_error(e.to_string()))?;

        // Collect rows — unify param binding to avoid closure type mismatch
        let dev_id_str = device_id.unwrap_or_default();
        let mut query_rows = if !dev_id_str.is_empty() {
            stmt.query(rusqlite::params![from_date, to_date, dev_id_str])
        } else {
            stmt.query(rusqlite::params![from_date, to_date])
        }.map_err(|e| internal_error(e.to_string()))?;

        let mut data: Vec<(String, f64)> = Vec::new();
        let mut total_seconds: f64 = 0.0;
        while let Some(row) = query_rows.next().map_err(|e| internal_error(e.to_string()))? {
            let name: String = row.get(0).map_err(|e| internal_error(e.to_string()))?;
            let secs: f64 = row.get(1).map_err(|e| internal_error(e.to_string()))?;
            total_seconds += secs;
            data.push((name, secs));
        }

        let groups: Vec<serde_json::Value> = data
            .into_iter()
            .map(|(name, secs)| {
                let pct = if total_seconds > 0.0 {
                    (secs / total_seconds * 1000.0).round() / 10.0
                } else {
                    0.0
                };
                serde_json::json!({
                    "name": name,
                    "seconds": secs.round() as i64,
                    "time": format_duration(secs),
                    "pct": pct,
                })
            })
            .collect();

        Ok(Json(serde_json::json!({
            "period_from": from_date,
            "period_to": to_date,
            "total_active": format_duration(total_seconds),
            "total_active_seconds": total_seconds.round() as i64,
            "groups": groups,
        })))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_current() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)>
{
    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let current =
            query::current::get_current(&conn).map_err(|e| internal_error(e.to_string()))?;
        match current {
            Some(c) => Ok(Json(serde_json::to_value(c).unwrap())),
            None => Ok(Json(serde_json::Value::Null)),
        }
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_categories(
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let cats = categories::list_categories(&conn).map_err(|e| internal_error(e.to_string()))?;
        Ok(Json(serde_json::to_value(cats).unwrap()))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_apps(
    Query(params): Query<AppsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (from_date, to_date) = date_range(params.from, params.to);
    let limit = params.limit.unwrap_or(20);

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let apps = query::apps::build_apps(&conn, &from_date, &to_date, limit)
            .map_err(|e| internal_error(e.to_string()))?;
        Ok(Json(serde_json::to_value(apps).unwrap()))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_timeline(
    Query(params): Query<TimelineParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (from_date, to_date) = date_range(params.from, params.to);
    let limit = params.limit.unwrap_or(200);
    let device_filter = params.device;

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let device_id = resolve_device_id(&conn, &device_filter);
        let device_clause = if device_id.is_some() {
            " AND e.device_id = ?4"
        } else {
            ""
        };

        let sql = format!(
            "SELECT e.timestamp, e.duration, e.app, e.title, e.url,
                    COALESCE(c.name, 'uncategorized') as category, e.is_afk
             FROM events e
             LEFT JOIN categories c ON e.category_id = c.id
             WHERE e.timestamp >= ?1 AND e.timestamp <= ?2 AND e.is_afk = 0 AND e.duration > 0{}
             ORDER BY e.timestamp ASC
             LIMIT ?3",
            device_clause,
        );

        let mut stmt = conn.prepare(&sql).map_err(|e| internal_error(e.to_string()))?;

        // Unify param binding to avoid closure type mismatch
        let dev_id_str = device_id.unwrap_or_default();
        let mut query_rows = if !dev_id_str.is_empty() {
            stmt.query(rusqlite::params![from_date, to_date, limit, dev_id_str])
        } else {
            stmt.query(rusqlite::params![from_date, to_date, limit])
        }.map_err(|e| internal_error(e.to_string()))?;

        let mut timeline = Vec::new();
        while let Some(row) = query_rows.next().map_err(|e| internal_error(e.to_string()))? {
            let ts: String = row.get(0).map_err(|e| internal_error(e.to_string()))?;
            let dur: f64 = row.get(1).map_err(|e| internal_error(e.to_string()))?;
            let app: Option<String> = row.get(2).map_err(|e| internal_error(e.to_string()))?;
            let title: Option<String> = row.get(3).map_err(|e| internal_error(e.to_string()))?;
            let url: Option<String> = row.get(4).map_err(|e| internal_error(e.to_string()))?;
            let category: String = row.get(5).map_err(|e| internal_error(e.to_string()))?;
            let is_afk: i32 = row.get(6).map_err(|e| internal_error(e.to_string()))?;
            timeline.push(serde_json::json!({
                "timestamp": ts,
                "duration": dur,
                "app": app,
                "title": title,
                "url": url,
                "category": category,
                "is_afk": is_afk != 0,
            }));
        }

        Ok(Json(serde_json::Value::Array(timeline)))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_productivity(
    Query(params): Query<TimeRangeParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (from_date, to_date) = date_range(params.from, params.to);

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let prod = query::productivity::build_productivity(&conn, &from_date, &to_date)
            .map_err(|e| internal_error(e.to_string()))?;
        Ok(Json(serde_json::to_value(prod).unwrap()))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_trends(
    Query(params): Query<TrendsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (from_date, to_date) = date_range(params.from, params.to);
    let interval = params.interval.unwrap_or_else(|| "day".into());

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let trends = query::trends::build_trends(&conn, &from_date, &to_date, &interval)
            .map_err(|e| internal_error(e.to_string()))?;
        Ok(Json(serde_json::to_value(trends).unwrap()))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_rules() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;
        let rules = categories::list_rules(&conn).map_err(|e| internal_error(e.to_string()))?;
        Ok(Json(serde_json::to_value(rules).unwrap()))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn post_rule(
    Json(body): Json<RuleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pattern = match body.app {
        Some(a) => a,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing required fields: app, category_id, field" })),
            ))
        }
    };
    let category_id = match body.category_id {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing required fields: app, category_id, field" })),
            ))
        }
    };
    let field = body.field.unwrap_or_else(|| "app".into());
    if !["app", "title", "url_domain"].contains(&field.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "field must be one of: app, title, url_domain" })),
        ));
    }

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;

        // Upsert: check for existing user rule
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM category_rules WHERE field = ?1 AND pattern = ?2 AND is_builtin = 0",
                rusqlite::params![field, pattern],
                |row| row.get(0),
            )
            .ok();

        if let Some(existing_id) = existing {
            conn.execute(
                "UPDATE category_rules SET category_id = ?1 WHERE id = ?2",
                rusqlite::params![category_id, existing_id],
            )
            .map_err(|e| internal_error(e.to_string()))?;
        } else {
            conn.execute(
                "INSERT INTO category_rules (category_id, field, pattern, is_builtin, priority) VALUES (?1, ?2, ?3, 0, 100)",
                rusqlite::params![category_id, field, pattern],
            )
            .map_err(|e| internal_error(e.to_string()))?;
        }

        // Recategorize matching events
        let updated = recategorize_events(&conn, &field, &pattern, category_id)
            .map_err(|e| internal_error(e.to_string()))?;

        Ok(Json(serde_json::json!({ "success": true, "updated": updated })))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn put_rule(
    Path(rule_id): Path<i64>,
    Json(body): Json<UpdateRuleBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let category_id = match body.category_id {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing required field: category_id" })),
            ))
        }
    };

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;

        let rule: Option<(String, String, bool)> = conn
            .query_row(
                "SELECT field, pattern, is_builtin FROM category_rules WHERE id = ?1",
                rusqlite::params![rule_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i32>(2)? != 0)),
            )
            .ok();

        let (field, pattern, is_builtin) = match rule {
            Some(r) => r,
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Rule not found" })),
                ))
            }
        };

        if is_builtin {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Cannot modify builtin rules" })),
            ));
        }

        conn.execute(
            "UPDATE category_rules SET category_id = ?1 WHERE id = ?2",
            rusqlite::params![category_id, rule_id],
        )
        .map_err(|e| internal_error(e.to_string()))?;

        let updated = recategorize_events(&conn, &field, &pattern, category_id)
            .map_err(|e| internal_error(e.to_string()))?;

        Ok(Json(serde_json::json!({ "success": true, "updated": updated })))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn delete_rule(
    Path(rule_id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;

        let rule: Option<(String, String, bool)> = conn
            .query_row(
                "SELECT field, pattern, is_builtin FROM category_rules WHERE id = ?1",
                rusqlite::params![rule_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i32>(2)? != 0)),
            )
            .ok();

        let (field, pattern, is_builtin) = match rule {
            Some(r) => r,
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Rule not found" })),
                ))
            }
        };

        if is_builtin {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Cannot delete builtin rules" })),
            ));
        }

        // Find matching builtin rule for fallback
        let builtin_cat: Option<i64> = conn
            .query_row(
                "SELECT category_id FROM category_rules WHERE field = ?1 AND pattern = ?2 AND is_builtin = 1 LIMIT 1",
                rusqlite::params![field, pattern],
                |row| row.get(0),
            )
            .ok();

        let fallback_cat = match builtin_cat {
            Some(id) => Some(id),
            None => conn
                .query_row(
                    "SELECT id FROM categories WHERE name = 'uncategorized' LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .ok(),
        };

        // Recategorize affected events
        let recategorized = recategorize_events_nullable(&conn, &field, &pattern, fallback_cat)
            .map_err(|e| internal_error(e.to_string()))?;

        // Delete the rule
        conn.execute(
            "DELETE FROM category_rules WHERE id = ?1",
            rusqlite::params![rule_id],
        )
        .map_err(|e| internal_error(e.to_string()))?;

        Ok(Json(serde_json::json!({ "success": true, "recategorized": recategorized })))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

pub async fn get_app_details(
    Path(name): Path<String>,
    Query(params): Query<TimeRangeParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let (from_date, to_date) = date_range(params.from, params.to);

    tokio::task::spawn_blocking(move || {
        let conn = db::open_default_db().map_err(|e| internal_error(e.to_string()))?;

        // Match on either app name or url_domain
        let mut stmt = conn
            .prepare(
                "SELECT e.timestamp, e.duration, e.app, e.title, e.url,
                        COALESCE(c.name, 'uncategorized') as category
                 FROM events e
                 LEFT JOIN categories c ON e.category_id = c.id
                 WHERE e.timestamp >= ?1 AND e.timestamp <= ?2
                   AND e.is_afk = 0 AND e.duration > 0
                   AND (e.app = ?3 OR e.url_domain = ?3)
                 ORDER BY e.timestamp ASC
                 LIMIT 200",
            )
            .map_err(|e| internal_error(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![from_date, to_date, name], |row| {
                Ok(serde_json::json!({
                    "timestamp": row.get::<_, String>(0)?,
                    "duration": row.get::<_, f64>(1)?,
                    "app": row.get::<_, Option<String>>(2)?,
                    "title": row.get::<_, Option<String>>(3)?,
                    "url": row.get::<_, Option<String>>(4)?,
                    "category": row.get::<_, String>(5)?,
                }))
            })
            .map_err(|e| internal_error(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|e| internal_error(e.to_string()))?);
        }

        Ok(Json(serde_json::json!({
            "app": name,
            "sessions": sessions,
        })))
    })
    .await
    .map_err(|e| internal_error(e.to_string()))?
}

fn recategorize_events(
    conn: &rusqlite::Connection,
    field: &str,
    pattern: &str,
    category_id: i64,
) -> std::result::Result<usize, String> {
    let changed = match field {
        "app" => conn
            .execute(
                "UPDATE events SET category_id = ?1 WHERE app = ?2 AND is_afk = 0",
                rusqlite::params![category_id, pattern],
            )
            .map_err(|e| e.to_string())?,
        "url_domain" => conn
            .execute(
                "UPDATE events SET category_id = ?1 WHERE url_domain = ?2 AND is_afk = 0",
                rusqlite::params![category_id, pattern],
            )
            .map_err(|e| e.to_string())?,
        _ => conn
            .execute(
                "UPDATE events SET category_id = ?1 WHERE title LIKE ?2 AND is_afk = 0",
                rusqlite::params![category_id, format!("%{}%", pattern)],
            )
            .map_err(|e| e.to_string())?,
    };
    Ok(changed)
}

fn recategorize_events_nullable(
    conn: &rusqlite::Connection,
    field: &str,
    pattern: &str,
    category_id: Option<i64>,
) -> std::result::Result<usize, String> {
    let changed = match field {
        "app" => conn
            .execute(
                "UPDATE events SET category_id = ?1 WHERE app = ?2 AND is_afk = 0",
                rusqlite::params![category_id, pattern],
            )
            .map_err(|e| e.to_string())?,
        "url_domain" => conn
            .execute(
                "UPDATE events SET category_id = ?1 WHERE url_domain = ?2 AND is_afk = 0",
                rusqlite::params![category_id, pattern],
            )
            .map_err(|e| e.to_string())?,
        _ => conn
            .execute(
                "UPDATE events SET category_id = ?1 WHERE title LIKE ?2 AND is_afk = 0",
                rusqlite::params![category_id, format!("%{}%", pattern)],
            )
            .map_err(|e| e.to_string())?,
    };
    Ok(changed)
}

// --- Embedded SPA serving ---

pub async fn serve_embedded(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file
    if let Some(content) = DashboardAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        (
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data.into_owned(),
        )
            .into_response()
    } else if let Some(content) = DashboardAssets::get("index.html") {
        // SPA fallback
        Html(String::from_utf8_lossy(&content.data).into_owned()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Dashboard not built. Run: make build").into_response()
    }
}
