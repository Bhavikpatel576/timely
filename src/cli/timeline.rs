use crate::db;
use crate::db::config_store;
use crate::error::Result;
use crate::output;
use crate::query;
use crate::query::timeline;
use crate::sync::client;

pub fn cmd_timeline(from: &str, to: &str, limit: Option<i64>, json: bool, all_devices: bool, device: Option<&str>) -> Result<()> {
    // Remote query mode: when --all-devices or --device is set
    if all_devices || device.is_some() {
        let conn = db::open_default_db()?;
        let hub_url = config_store::get(&conn, "sync.hub_url")?
            .ok_or_else(|| crate::error::TimelyError::Config("sync.hub_url not configured. Run: timely sync setup".into()))?;
        let api_key = config_store::get(&conn, "sync.api_key")?;

        let device_param = if all_devices { Some("all") } else { device };
        let result = client::fetch_remote_timeline(&hub_url, &api_key, from, to, limit, device_param)?;

        if json {
            println!("{}", serde_json::to_string_pretty(&output::success(&result)).unwrap());
        } else {
            println!("Timeline ({} to {}) [remote]", from, to);
            println!("{:-<70}", "");
            if let Some(entries) = result.get("entries").and_then(|e| e.as_array()) {
                for entry in entries {
                    let ts = entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or("?");
                    let app = entry.get("app").and_then(|v| v.as_str()).unwrap_or("?");
                    let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                    let dur = entry.get("duration").or_else(|| entry.get("duration_seconds"))
                        .and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let cat = entry.get("category").and_then(|v| v.as_str()).unwrap_or("-");
                    let time_str = if ts.len() >= 19 { &ts[11..19] } else { ts };
                    println!("{} {:>8}  {:<20} {:<30} {}",
                        time_str,
                        crate::types::format_duration(dur),
                        app,
                        truncate(title, 30),
                        cat,
                    );
                }
                println!("\n{} entries", entries.len());
            }
        }
        return Ok(());
    }

    // Local query mode (unchanged)
    let from_dt = query::parse_time(from)?;
    let to_dt = query::parse_time(to)?;

    let conn = db::open_default_db()?;
    let result = timeline::build_timeline(&conn, &from_dt, &to_dt, limit)?;

    if json {
        output::print_json(&result);
    } else {
        println!("Timeline ({} to {})", from, to);
        println!("{:-<70}", "");
        for entry in &result.entries {
            let cat = entry.category.as_deref().unwrap_or("-");
            let afk = if entry.is_afk { " [AFK]" } else { "" };
            println!(
                "{} {:>8}  {:<20} {:<30} {}{}",
                &entry.timestamp[11..19],
                entry.duration_time,
                entry.app,
                truncate(&entry.title, 30),
                cat,
                afk,
            );
        }
        println!("\n{} entries", result.count);
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
