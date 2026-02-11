use crate::db;
use crate::db::config_store;
use crate::error::Result;
use crate::output;
use crate::query;
use crate::query::summary::{self, GroupBy};
use crate::sync::client;

pub fn cmd_summary(from: &str, to: &str, by: &str, json: bool, all_devices: bool, device: Option<&str>) -> Result<()> {
    // Remote query mode: when --all-devices or --device is set
    if all_devices || device.is_some() {
        let conn = db::open_default_db()?;
        let hub_url = config_store::get(&conn, "sync.hub_url")?
            .ok_or_else(|| crate::error::TimelyError::Config("sync.hub_url not configured. Run: timely sync setup".into()))?;
        let api_key = config_store::get(&conn, "sync.api_key")?;

        let device_param = if all_devices { Some("all") } else { device };
        let result = client::fetch_remote_summary(&hub_url, &api_key, from, to, by, device_param)?;

        // Print the response directly — it's already formatted from the hub
        println!("{}", serde_json::to_string_pretty(&output::success(&result)).unwrap());
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
    let result = summary::build_summary(&conn, &from_dt, &to_dt, group_by)?;

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
