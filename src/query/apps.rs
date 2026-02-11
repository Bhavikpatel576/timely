use rusqlite::Connection;
use crate::error::Result;
use crate::types::{format_duration, AppBreakdown};

pub fn build_apps(
    conn: &Connection,
    from_date: &str,
    to_date: &str,
    limit: i64,
) -> Result<Vec<AppBreakdown>> {
    let mut stmt = conn.prepare(
        "SELECT
           CASE
             WHEN e.url_domain IS NOT NULL AND e.url_domain != ''
             THEN e.url_domain
             ELSE COALESCE(e.app, 'Unknown')
           END as label,
           COALESCE(c.name, 'uncategorized') as category,
           SUM(e.duration) as total_seconds,
           COUNT(*) as event_count
         FROM events e
         LEFT JOIN categories c ON e.category_id = c.id
         WHERE e.timestamp >= ?1 AND e.timestamp <= ?2 AND e.is_afk = 0 AND e.duration > 0
         GROUP BY label
         ORDER BY total_seconds DESC
         LIMIT ?3",
    )?;

    let rows = stmt.query_map(rusqlite::params![from_date, to_date, limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, i64>(3)?,
        ))
    })?;

    let mut data = Vec::new();
    let mut total_seconds: f64 = 0.0;
    for row in rows {
        let r = row?;
        total_seconds += r.2;
        data.push(r);
    }

    let apps = data
        .into_iter()
        .map(|(app, category, secs, events)| {
            let pct = if total_seconds > 0.0 {
                (secs / total_seconds * 1000.0).round() / 10.0
            } else {
                0.0
            };
            AppBreakdown {
                app,
                category,
                seconds: secs.round() as i64,
                time: format_duration(secs),
                pct,
                events,
            }
        })
        .collect();

    Ok(apps)
}
