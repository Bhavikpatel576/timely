use crate::db;
use crate::error::Result;
use crate::output;
use crate::query;
use crate::query::trends;
use crate::types::TrendsResponse;

pub fn cmd_trends(from: &str, to: &str, interval: &str, json: bool) -> Result<()> {
    let from_dt = query::parse_time(from)?;
    let to_dt = query::parse_time(to)?;

    let conn = db::open_default_db()?;
    let buckets = trends::build_trends(
        &conn,
        &from_dt.to_rfc3339(),
        &to_dt.to_rfc3339(),
        interval,
    )?;

    let result = TrendsResponse {
        from: from_dt.to_rfc3339(),
        to: to_dt.to_rfc3339(),
        interval: interval.to_string(),
        buckets,
    };

    if json {
        output::print_json(&result);
    } else {
        println!("Trends ({} to {}, interval: {})", from, to, interval);
        println!("{:-<60}", "");
        for bucket in &result.buckets {
            println!(
                "{:<20} {:>6.1}h  productivity: {}%",
                bucket.bucket, bucket.total_hours, bucket.productivity
            );
        }
        if result.buckets.is_empty() {
            println!("No data for the requested time range.");
        }
    }

    Ok(())
}
