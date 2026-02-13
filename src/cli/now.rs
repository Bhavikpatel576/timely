use chrono::Utc;
use crate::config::HEARTBEAT_MERGE_GAP_SECS;
use crate::db;
use crate::db::config_store;
use crate::db::devices;
use crate::error::{Result, TimelyError};
use crate::output;
use crate::sync::client;
use crate::types::{format_duration, NowResponse};

pub fn cmd_now(json: bool, all_devices: bool, device: Option<&str>) -> Result<()> {
    // Remote query mode: when --all-devices or --device is set
    if all_devices || device.is_some() {
        let conn = db::open_default_db()?;
        let hub_url = config_store::get(&conn, "sync.hub_url")?
            .ok_or_else(|| TimelyError::Config("sync.hub_url not configured. Run: timely sync setup".into()))?;
        let api_key = config_store::get(&conn, "sync.api_key")?;

        let device_param = if all_devices { Some("all") } else { device };
        let result = client::fetch_remote_now(&hub_url, &api_key, device_param)?;

        if json {
            println!("{}", serde_json::to_string_pretty(&output::success(&result)).unwrap());
        } else {
            let app = result.get("app").and_then(|v| v.as_str()).unwrap_or("?");
            let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("?");
            let since = result.get("since").and_then(|v| v.as_str()).unwrap_or("?");
            let dur = result.get("duration_seconds").and_then(|v| v.as_f64()).unwrap_or(0.0);
            println!("{} — {}", app, title);
            println!("Since: {} ({})", since, format_duration(dur));
        }
        return Ok(());
    }

    // Local query mode
    let conn = db::open_default_db()?;
    let device = devices::get_or_create_device(&conn)?;

    // Find the most recent event with a non-empty app name
    let last = {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.device_id, e.timestamp, e.duration, e.app, e.title, e.url, e.url_domain,
                    e.category_id, c.name, e.is_afk
             FROM events e
             LEFT JOIN categories c ON c.id = e.category_id
             WHERE e.device_id = ?1 AND e.app != ''
             ORDER BY e.id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(rusqlite::params![device.id])?;
        match rows.next()? {
            Some(row) => {
                let ts_str: String = row.get(2)?;
                let timestamp = chrono::DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                crate::types::Event {
                    id: row.get(0)?,
                    device_id: row.get(1)?,
                    timestamp,
                    duration: row.get(3)?,
                    app: row.get(4)?,
                    title: row.get(5)?,
                    url: row.get(6)?,
                    url_domain: row.get(7)?,
                    category_id: row.get(8)?,
                    category_name: row.get(9)?,
                    is_afk: row.get::<_, i32>(10)? != 0,
                }
            }
            None => return Err(TimelyError::NoData),
        }
    };

    let elapsed = (Utc::now() - last.timestamp).num_seconds() as f64;

    // If the last event is older than its duration + merge gap, the daemon
    // has stopped — use the stored duration instead of elapsed time.
    let active_duration = if elapsed > last.duration + HEARTBEAT_MERGE_GAP_SECS {
        last.duration
    } else {
        last.duration.max(elapsed)
    };

    let stale = elapsed > last.duration + HEARTBEAT_MERGE_GAP_SECS;

    let response = NowResponse {
        app: last.app,
        title: last.title,
        url: last.url,
        category: last.category_name,
        productivity_score: None,
        since: last.timestamp.to_rfc3339(),
        duration_seconds: active_duration,
        duration_time: format_duration(active_duration),
        is_afk: last.is_afk,
        stale,
    };

    if json {
        output::print_json(&response);
    } else {
        println!("{} — {}", response.app, response.title);
        if let Some(ref url) = response.url {
            println!("URL: {}", url);
        }
        if let Some(ref cat) = response.category {
            println!("Category: {}", cat);
        }
        println!("Since: {} ({})", response.since, response.duration_time);
        if response.is_afk {
            println!("Status: AFK");
        }
        if response.stale {
            println!("Note: daemon is not running — showing last recorded activity");
        }
    }

    Ok(())
}
