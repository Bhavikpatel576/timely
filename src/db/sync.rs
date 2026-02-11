use rusqlite::Connection;
use crate::error::Result;

pub fn upsert_remote_event(
    conn: &Connection,
    device_id: &str,
    timestamp: &str,
    duration: f64,
    app: &str,
    title: &str,
    url: Option<&str>,
    url_domain: Option<&str>,
    category_id: Option<i64>,
    is_afk: bool,
) -> Result<bool> {
    // Dedup key: (device_id, timestamp, app, title). On conflict, take MAX(duration).
    let existing: Option<(i64, f64)> = conn
        .query_row(
            "SELECT id, duration FROM events
             WHERE device_id = ?1 AND timestamp = ?2 AND app = ?3 AND title = ?4",
            rusqlite::params![device_id, timestamp, app, title],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    if let Some((id, existing_dur)) = existing {
        if duration > existing_dur {
            conn.execute(
                "UPDATE events SET duration = ?1, url = ?2, url_domain = ?3,
                        category_id = ?4, is_afk = ?5 WHERE id = ?6",
                rusqlite::params![duration, url, url_domain, category_id, is_afk as i32, id],
            )?;
        }
        Ok(false) // duplicate
    } else {
        conn.execute(
            "INSERT INTO events (device_id, timestamp, duration, app, title, url, url_domain, category_id, is_afk)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![device_id, timestamp, duration, app, title, url, url_domain, category_id, is_afk as i32],
        )?;
        Ok(true) // new
    }
}

pub fn upsert_remote_device(
    conn: &Connection,
    id: &str,
    name: &str,
    platform: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO devices (id, name, platform, last_sync)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            platform = excluded.platform,
            last_sync = datetime('now')",
        rusqlite::params![id, name, platform],
    )?;
    Ok(())
}

pub fn get_sync_log(conn: &Connection, device_id: &str) -> Result<Option<(i64, String)>> {
    let result = conn
        .query_row(
            "SELECT last_synced_event_id, last_sync_at FROM sync_log WHERE device_id = ?1",
            rusqlite::params![device_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    Ok(result)
}

pub fn update_sync_log(conn: &Connection, device_id: &str, last_event_id: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO sync_log (device_id, last_synced_event_id, last_sync_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(device_id) DO UPDATE SET
            last_synced_event_id = excluded.last_synced_event_id,
            last_sync_at = datetime('now')",
        rusqlite::params![device_id, last_event_id],
    )?;
    Ok(())
}

pub fn get_device_event_counts(conn: &Connection) -> Result<Vec<(String, String, String, Option<String>, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.platform, d.last_sync,
                (SELECT COUNT(*) FROM events WHERE device_id = d.id) as event_count
         FROM devices d
         ORDER BY d.name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_total_event_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;
    Ok(count)
}
