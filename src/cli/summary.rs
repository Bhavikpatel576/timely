use crate::db;
use crate::db::config_store;
use crate::error::Result;
use crate::output;
use crate::query;
use crate::query::summary::{self, GroupBy};
use crate::sync::client;

pub fn cmd_summary(from: &str, to: &str, by: &str, exclude_afk: bool, json: bool, all_devices: bool, device: Option<&str>) -> Result<()> {
    // Remote query mode: when --all-devices or --device is set
    if all_devices || device.is_some() {
        let conn = db::open_default_db()?;
        let hub_url = config_store::get(&conn, "sync.hub_url")?
            .ok_or_else(|| crate::error::TimelyError::Config("sync.hub_url not configured. Run: timely sync setup".into()))?;
        let api_key = config_store::get(&conn, "sync.api_key")?;

        let device_param = if all_devices { Some("all") } else { device };
        let result = client::fetch_remote_summary(&hub_url, &api_key, from, to, by, exclude_afk, device_param)?;

        if json {
            println!("{}", serde_json::to_string_pretty(&output::success(&result)).unwrap());
        } else {
            println!("Activity Summary ({} to {}) [remote]", from, to);
            if let Some(groups) = result.get("groups").and_then(|g| g.as_array()) {
                println!("{:-<60}", "");
                for g in groups {
                    let label = g.get("name").or_else(|| g.get("label"))
                        .and_then(|v| v.as_str()).unwrap_or("?");
                    let time = g.get("time").and_then(|v| v.as_str()).unwrap_or("?");
                    let pct = g.get("pct").or_else(|| g.get("percentage"))
                        .and_then(|v| v.as_f64()).unwrap_or(0.0);
                    println!("{:<30} {:>8}  {:>5.1}%", label, time, pct);
                }
            }
        }
        return Ok(());
    }

    // Local query mode (unchanged)
    let from_dt = query::parse_time(from)?;
    let to_dt = query::parse_time(to)?;

    let group_by = match by {
        "app" => GroupBy::App,
        "url" => GroupBy::Url,
        _ => GroupBy::Category,
    };

    let conn = db::open_default_db()?;
    let result = summary::build_summary(&conn, &from_dt, &to_dt, group_by, exclude_afk)?;

    if json {
        output::print_json(&result);
    } else {
        println!("Activity Summary ({} to {})", from, to);
        println!("Total: {} | Productivity: {:.2}", result.total_time, result.productivity_score);
        println!("{:-<60}", "");
        for group in &result.groups {
            let score_str = group
                .productivity_score
                .map(|s| format!(" [{:+.1}]", s))
                .unwrap_or_default();
            println!(
                "{:<30} {:>8}  {:>5.1}%{}",
                group.label, group.time, group.percentage, score_str
            );
        }
    }

    Ok(())
}
