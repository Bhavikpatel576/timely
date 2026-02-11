use chrono::Utc;
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

        println!("{}", serde_json::to_string_pretty(&output::success(&result)).unwrap());
        return Ok(());
    }

    // Local query mode (unchanged)
    let conn = db::open_default_db()?;
    let device = devices::get_or_create_device(&conn)?;

    let last = crate::db::events::get_last_event(&conn, &device.id)?
        .ok_or(TimelyError::NoData)?;

    let elapsed = (Utc::now() - last.timestamp).num_seconds() as f64;
    let active_duration = last.duration.max(elapsed);

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
    }

    Ok(())
}
