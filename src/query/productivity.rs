use rusqlite::Connection;
use crate::error::Result;
use crate::types::ProductivityResponse;

pub fn build_productivity(
    conn: &Connection,
    from_date: &str,
    to_date: &str,
) -> Result<ProductivityResponse> {
    let mut stmt = conn.prepare(
        "SELECT
           COALESCE(c.productivity_score, 0) as score,
           SUM(e.duration) as total_seconds
         FROM events e
         LEFT JOIN categories c ON e.category_id = c.id
         WHERE e.timestamp >= ?1 AND e.timestamp <= ?2 AND e.is_afk = 0
         GROUP BY COALESCE(c.productivity_score, 0)",
    )?;

    let rows = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?))
    })?;

    let mut productive: f64 = 0.0;
    let mut neutral: f64 = 0.0;
    let mut distracting: f64 = 0.0;
    let mut weighted_sum: f64 = 0.0;
    let mut total_seconds: f64 = 0.0;

    for row in rows {
        let (score, secs) = row?;
        total_seconds += secs;
        if score > 0.0 {
            productive += secs;
            weighted_sum += secs * score;
        } else if score < 0.0 {
            distracting += secs;
            weighted_sum += secs * score;
        } else {
            neutral += secs;
        }
    }

    let weighted_avg = if total_seconds > 0.0 {
        weighted_sum / total_seconds
    } else {
        0.0
    };
    let score = ((weighted_avg + 2.0) / 4.0 * 100.0).round() as i64;

    Ok(ProductivityResponse {
        score: score.clamp(0, 100),
        productive: productive.round() as i64,
        neutral: neutral.round() as i64,
        distracting: distracting.round() as i64,
        total: total_seconds.round() as i64,
    })
}
