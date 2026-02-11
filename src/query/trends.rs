use std::collections::HashMap;
use rusqlite::Connection;
use crate::error::Result;
use crate::types::TrendBucket;

pub fn build_trends(
    conn: &Connection,
    from_date: &str,
    to_date: &str,
    interval: &str,
) -> Result<Vec<TrendBucket>> {
    let date_expr = match interval {
        "hour" => "strftime('%Y-%m-%dT%H:00', e.timestamp, 'localtime')",
        "week" => "strftime('%Y-W%W', e.timestamp, 'localtime')",
        "month" => "strftime('%Y-%m', e.timestamp, 'localtime')",
        _ => "date(e.timestamp, 'localtime')",
    };

    let sql = format!(
        "SELECT
           {} as bucket,
           COALESCE(c.name, 'uncategorized') as category,
           COALESCE(c.productivity_score, 0) as prod_score,
           SUM(e.duration) as total_seconds
         FROM events e
         LEFT JOIN categories c ON e.category_id = c.id
         WHERE e.timestamp >= ?1 AND e.timestamp <= ?2 AND e.is_afk = 0
         GROUP BY bucket, category
         ORDER BY bucket ASC",
        date_expr
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;

    // Pivot into per-bucket objects (preserving insertion order via Vec)
    let mut bucket_order: Vec<String> = Vec::new();
    let mut bucket_map: HashMap<String, BucketAccum> = HashMap::new();

    for row in rows {
        let (bucket, category, prod_score, total_secs) = row?;
        let entry = bucket_map.entry(bucket.clone()).or_insert_with(|| {
            bucket_order.push(bucket);
            BucketAccum::default()
        });
        entry.total += total_secs;
        *entry.categories.entry(category).or_insert(0.0) += total_secs;
        entry.weighted_sum += total_secs * prod_score;
    }

    let trends = bucket_order
        .into_iter()
        .map(|bucket| {
            let entry = &bucket_map[&bucket];
            let productivity = if entry.total > 0.0 {
                ((entry.weighted_sum / entry.total + 2.0) / 4.0 * 100.0)
                    .round()
                    .clamp(0.0, 100.0) as i64
            } else {
                50
            };
            TrendBucket {
                bucket,
                total_seconds: entry.total.round() as i64,
                total_hours: (entry.total / 3600.0 * 10.0).round() / 10.0,
                productivity,
                categories: entry.categories.clone(),
            }
        })
        .collect();

    Ok(trends)
}

#[derive(Default)]
struct BucketAccum {
    total: f64,
    categories: HashMap<String, f64>,
    weighted_sum: f64,
}
