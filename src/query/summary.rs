use chrono::{DateTime, Utc};
use rusqlite::Connection;
use crate::error::{Result, TimelyError};
use crate::types::{format_duration, SummaryGroup, SummaryResponse};

#[derive(Debug, Clone, Copy)]
pub enum GroupBy {
    Category,
    App,
    Url,
}

pub fn build_summary(
    conn: &Connection,
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
    group_by: GroupBy,
) -> Result<SummaryResponse> {
    let sql = match group_by {
        GroupBy::Category =>
            "SELECT COALESCE(c.name, 'uncategorized') as grp, SUM(e.duration) as total_dur,
                    COUNT(*) as cnt, COALESCE(c.productivity_score, 0.0) as score
             FROM events e
             LEFT JOIN categories c ON c.id = e.category_id
             WHERE e.timestamp >= ?1 AND e.timestamp <= ?2
             GROUP BY grp
             ORDER BY total_dur DESC",
        GroupBy::App =>
            "SELECT e.app as grp, SUM(e.duration) as total_dur,
                    COUNT(*) as cnt, COALESCE(c.productivity_score, 0.0) as score
             FROM events e
             LEFT JOIN categories c ON c.id = e.category_id
             WHERE e.timestamp >= ?1 AND e.timestamp <= ?2
             GROUP BY e.app
             ORDER BY total_dur DESC",
        GroupBy::Url =>
            "SELECT COALESCE(e.url_domain, e.app) as grp, SUM(e.duration) as total_dur,
                    COUNT(*) as cnt, COALESCE(c.productivity_score, 0.0) as score
             FROM events e
             LEFT JOIN categories c ON c.id = e.category_id
             WHERE e.timestamp >= ?1 AND e.timestamp <= ?2
             GROUP BY grp
             ORDER BY total_dur DESC",
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(
        rusqlite::params![from.to_rfc3339(), to.to_rfc3339()],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        },
    )?;

    let mut groups = Vec::new();
    let mut total_seconds: f64 = 0.0;
    let mut weighted_score: f64 = 0.0;

    // Collect all rows
    let mut data: Vec<(String, f64, i64, f64)> = Vec::new();
    for row in rows {
        let (label, dur, cnt, score) = row?;
        total_seconds += dur;
        weighted_score += dur * score;
        data.push((label, dur, cnt, score));
    }

    if data.is_empty() {
        return Err(TimelyError::NoData);
    }

    let productivity_score = if total_seconds > 0.0 {
        weighted_score / total_seconds
    } else {
        0.0
    };

    // Build groups with percentages
    for (label, dur, cnt, score) in &data {
        let pct = if total_seconds > 0.0 {
            (*dur / total_seconds) * 100.0
        } else {
            0.0
        };
        groups.push(SummaryGroup {
            label: label.clone(),
            seconds: *dur,
            time: format_duration(*dur),
            percentage: (pct * 10.0).round() / 10.0,
            productivity_score: Some(*score),
            event_count: *cnt,
        });
    }

    Ok(SummaryResponse {
        from: from.to_rfc3339(),
        to: to.to_rfc3339(),
        total_seconds,
        total_time: format_duration(total_seconds),
        productivity_score: (productivity_score * 100.0).round() / 100.0,
        groups,
    })
}
