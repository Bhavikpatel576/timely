use chrono::{DateTime, Utc};
use rusqlite::Connection;
use crate::error::Result;
use crate::types::Event;

pub fn insert_event(
    conn: &Connection,
    device_id: &str,
    timestamp: &DateTime<Utc>,
    duration: f64,
    app: &str,
    title: &str,
    url: Option<&str>,
    url_domain: Option<&str>,
    category_id: Option<i64>,
    is_afk: bool,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO events (device_id, timestamp, duration, app, title, url, url_domain, category_id, is_afk)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            device_id,
            timestamp.to_rfc3339(),
            duration,
            app,
            title,
            url,
            url_domain,
            category_id,
            is_afk as i32,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn extend_event(conn: &Connection, event_id: i64, new_duration: f64) -> Result<()> {
    conn.execute(
        "UPDATE events SET duration = ?1 WHERE id = ?2",
        rusqlite::params![new_duration, event_id],
    )?;
    Ok(())
}

pub fn get_last_event(conn: &Connection, device_id: &str) -> Result<Option<Event>> {
    let mut stmt = conn.prepare(
        "SELECT e.id, e.device_id, e.timestamp, e.duration, e.app, e.title, e.url, e.url_domain,
                e.category_id, c.name, e.is_afk
         FROM events e
         LEFT JOIN categories c ON c.id = e.category_id
         WHERE e.device_id = ?1
         ORDER BY e.id DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(rusqlite::params![device_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(event_from_row(row)?))
    } else {
        Ok(None)
    }
}

pub fn query_events(
    conn: &Connection,
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
    limit: Option<i64>,
) -> Result<Vec<Event>> {
    let lim = limit.unwrap_or(i64::MAX);
    let sql = "SELECT e.id, e.device_id, e.timestamp, e.duration, e.app, e.title, e.url, e.url_domain,
                e.category_id, c.name, e.is_afk
         FROM events e
         LEFT JOIN categories c ON c.id = e.category_id
         WHERE e.timestamp >= ?1 AND e.timestamp <= ?2
         ORDER BY e.timestamp DESC
         LIMIT ?3";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(
        rusqlite::params![from.to_rfc3339(), to.to_rfc3339(), lim],
        |row| event_from_row(row),
    )?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn query_events_after_id(
    conn: &Connection,
    device_id: &str,
    after_id: i64,
    limit: i64,
) -> Result<Vec<Event>> {
    let sql = "SELECT e.id, e.device_id, e.timestamp, e.duration, e.app, e.title, e.url, e.url_domain,
                e.category_id, c.name, e.is_afk
         FROM events e
         LEFT JOIN categories c ON c.id = e.category_id
         WHERE e.device_id = ?1 AND e.id > ?2
         ORDER BY e.id ASC
         LIMIT ?3";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(
        rusqlite::params![device_id, after_id, limit],
        |row| event_from_row(row),
    )?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn update_event_category(conn: &Connection, event_id: i64, category_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE events SET category_id = ?1 WHERE id = ?2",
        rusqlite::params![category_id, event_id],
    )?;
    Ok(())
}

fn event_from_row(row: &rusqlite::Row) -> rusqlite::Result<Event> {
    let ts_str: String = row.get(2)?;
    let timestamp = DateTime::parse_from_rfc3339(&ts_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Event {
        id: row.get(0)?,
        device_id: row.get(1)?,
        timestamp,
        duration: row.get(3)?,
        app: row.get(4)?,
        title: row.get(5)?,
        url: row.get(6)?,
        url_domain: row.get(7)?,
        category_id: row.get(8)?,
        category_name: row.get(9)?,
        is_afk: row.get::<_, i32>(10)? != 0,
    })
}
