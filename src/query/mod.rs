pub mod summary;
pub mod timeline;
pub mod apps;
pub mod productivity;
pub mod trends;
pub mod current;

use chrono::{DateTime, Duration, Local, NaiveDate, Utc};
use crate::error::{Result, TimelyError};

/// Parse flexible time specifications:
/// - "now" → current time
/// - "today" → start of today (local)
/// - "yesterday" → start of yesterday (local)
/// - "Nd" (e.g. "7d") → N days ago
/// - "Nh" (e.g. "2h") → N hours ago
/// - "Nm" (e.g. "30m") → N minutes ago
/// - ISO 8601 date/datetime → parsed directly
pub fn parse_time(input: &str) -> Result<DateTime<Utc>> {
    let input = input.trim().to_lowercase();

    match input.as_str() {
        "now" => return Ok(Utc::now()),
        "today" => {
            let today = Local::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
            return Ok(today.and_local_timezone(Local).unwrap().with_timezone(&Utc));
        }
        "yesterday" => {
            let yesterday = (Local::now() - Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            return Ok(yesterday.and_local_timezone(Local).unwrap().with_timezone(&Utc));
        }
        _ => {}
    }

    // Relative: "7d", "2h", "30m"
    if let Some(num_str) = input.strip_suffix('d') {
        if let Ok(n) = num_str.parse::<i64>() {
            return Ok(Utc::now() - Duration::days(n));
        }
    }
    if let Some(num_str) = input.strip_suffix('h') {
        if let Ok(n) = num_str.parse::<i64>() {
            return Ok(Utc::now() - Duration::hours(n));
        }
    }
    if let Some(num_str) = input.strip_suffix('m') {
        if let Ok(n) = num_str.parse::<i64>() {
            return Ok(Utc::now() - Duration::minutes(n));
        }
    }

    // ISO 8601 datetime
    if let Ok(dt) = DateTime::parse_from_rfc3339(&input) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Date only (YYYY-MM-DD)
    if let Ok(date) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(dt.and_local_timezone(Local).unwrap().with_timezone(&Utc));
    }

    Err(TimelyError::InvalidTimeRange(format!(
        "Cannot parse '{}'. Use: now, today, yesterday, Nd, Nh, Nm, or YYYY-MM-DD",
        input
    )))
}
