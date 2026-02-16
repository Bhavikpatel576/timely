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
    exclude_afk: bool,
) -> Result<SummaryResponse> {
    let afk_filter = if exclude_afk { " AND e.is_afk = 0" } else { "" };

    let sql = match group_by {
        GroupBy::Category =>
            format!("SELECT COALESCE(c.name, 'uncategorized') as grp,
                            SUM(e.duration) as total_dur,
                            SUM(CASE WHEN e.is_afk = 0 THEN e.duration ELSE 0 END) as engaged_dur,
                            SUM(CASE WHEN e.is_afk = 1 THEN e.duration ELSE 0 END) as afk_dur,
                            COUNT(*) as cnt,
                            COALESCE(c.productivity_score, 0.0) as score
                     FROM events e
                     LEFT JOIN categories c ON c.id = e.category_id
                     WHERE e.timestamp >= ?1 AND e.timestamp <= ?2{}
                     GROUP BY grp
                     ORDER BY total_dur DESC", afk_filter),
        GroupBy::App =>
            format!("SELECT e.app as grp,
                            SUM(e.duration) as total_dur,
                            SUM(CASE WHEN e.is_afk = 0 THEN e.duration ELSE 0 END) as engaged_dur,
                            SUM(CASE WHEN e.is_afk = 1 THEN e.duration ELSE 0 END) as afk_dur,
                            COUNT(*) as cnt,
                            COALESCE(c.productivity_score, 0.0) as score
                     FROM events e
                     LEFT JOIN categories c ON c.id = e.category_id
                     WHERE e.timestamp >= ?1 AND e.timestamp <= ?2{}
                     GROUP BY e.app
                     ORDER BY total_dur DESC", afk_filter),
        GroupBy::Url =>
            format!("SELECT COALESCE(e.url_domain, e.app) as grp,
                            SUM(e.duration) as total_dur,
                            SUM(CASE WHEN e.is_afk = 0 THEN e.duration ELSE 0 END) as engaged_dur,
                            SUM(CASE WHEN e.is_afk = 1 THEN e.duration ELSE 0 END) as afk_dur,
                            COUNT(*) as cnt,
                            COALESCE(c.productivity_score, 0.0) as score
                     FROM events e
                     LEFT JOIN categories c ON c.id = e.category_id
                     WHERE e.timestamp >= ?1 AND e.timestamp <= ?2{}
                     GROUP BY grp
                     ORDER BY total_dur DESC", afk_filter),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params![from.to_rfc3339(), to.to_rfc3339()],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, f64>(5)?,
            ))
        },
    )?;

    let mut groups = Vec::new();
    let mut total_seconds: f64 = 0.0;
    let mut engaged_total_seconds: f64 = 0.0;
    let mut afk_total_seconds: f64 = 0.0;
    let mut weighted_score: f64 = 0.0;

    // Collect all rows
    let mut data: Vec<(String, f64, f64, f64, i64, f64)> = Vec::new();
    for row in rows {
        let (label, dur, engaged_dur, afk_dur, cnt, score) = row?;
        total_seconds += dur;
        engaged_total_seconds += engaged_dur;
        afk_total_seconds += afk_dur;
        weighted_score += dur * score;
        data.push((label, dur, engaged_dur, afk_dur, cnt, score));
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
    for (label, dur, engaged_dur, afk_dur, cnt, score) in &data {
        let pct = if total_seconds > 0.0 {
            (*dur / total_seconds) * 100.0
        } else {
            0.0
        };
        groups.push(SummaryGroup {
            label: label.clone(),
            seconds: *dur,
            time: format_duration(*dur),
            engaged_seconds: *engaged_dur,
            engaged_time: format_duration(*engaged_dur),
            afk_seconds: *afk_dur,
            afk_time: format_duration(*afk_dur),
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
        engaged_total_seconds,
        engaged_total_time: format_duration(engaged_total_seconds),
        afk_total_seconds,
        afk_total_time: format_duration(afk_total_seconds),
        productivity_score: (productivity_score * 100.0).round() / 100.0,
        groups,
    })
}
