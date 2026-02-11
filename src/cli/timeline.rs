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

        println!("{}", serde_json::to_string_pretty(&output::success(&result)).unwrap());
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
