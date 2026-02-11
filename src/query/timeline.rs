use chrono::{DateTime, Utc};
use rusqlite::Connection;
use crate::db::events;
use crate::error::{Result, TimelyError};
use crate::types::{format_duration, TimelineEntry, TimelineResponse};

pub fn build_timeline(
    conn: &Connection,
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
    limit: Option<i64>,
) -> Result<TimelineResponse> {
    let mut event_list = events::query_events(conn, from, to, limit)?;

    if event_list.is_empty() {
        return Err(TimelyError::NoData);
    }

    // Reverse to chronological order (query returns DESC)
    event_list.reverse();

    let entries: Vec<TimelineEntry> = event_list
        .iter()
        .map(|e| TimelineEntry {
            timestamp: e.timestamp.to_rfc3339(),
            duration_seconds: e.duration,
            duration_time: format_duration(e.duration),
            app: e.app.clone(),
            title: e.title.clone(),
            url: e.url.clone(),
            category: e.category_name.clone(),
            productivity_score: None, // Could join if needed
            is_afk: e.is_afk,
        })
        .collect();

    Ok(TimelineResponse {
        from: from.to_rfc3339(),
        to: to.to_rfc3339(),
        count: entries.len(),
        entries,
    })
}
