use rusqlite::Connection;
use crate::error::Result;
use crate::types::CurrentActivity;

pub fn get_current(conn: &Connection) -> Result<Option<CurrentActivity>> {
    let mut stmt = conn.prepare(
        "SELECT e.app, e.title, e.url, COALESCE(c.name, 'uncategorized') as category,
                e.duration, e.is_afk, e.timestamp
         FROM events e
         LEFT JOIN categories c ON e.category_id = c.id
         ORDER BY e.timestamp DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(CurrentActivity {
            app: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            category: row.get(3)?,
            duration_seconds: row.get(4)?,
            is_afk: row.get::<_, i32>(5)? != 0,
            since: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}
