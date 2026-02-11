use chrono::Utc;
use rusqlite::Connection;
use crate::config::HEARTBEAT_MERGE_GAP_SECS;
use crate::error::Result;
use crate::types::WatcherSnapshot;
use crate::db::{events, categories as db_categories};
use crate::categories;

pub fn process_heartbeat(
    conn: &Connection,
    device_id: &str,
    snapshot: &WatcherSnapshot,
) -> Result<()> {
    let now = Utc::now();

    // Load rules for classification
    let rules = db_categories::list_rules(conn)?;
    let category_id = categories::classify(snapshot, &rules);

    // If no category matched, look up "uncategorized"
    let category_id = category_id.or_else(|| {
        db_categories::get_category_by_name(conn, "uncategorized")
            .ok()
            .flatten()
            .map(|c| c.id)
    });

    // Try to extend last event
    if let Some(last) = events::get_last_event(conn, device_id)? {
        let same_activity = last.app == snapshot.app
            && last.title == snapshot.title
            && last.url_domain == snapshot.url_domain
            && last.is_afk == snapshot.is_afk;

        if same_activity {
            let elapsed = (now - last.timestamp).num_milliseconds() as f64 / 1000.0;
            if elapsed < last.duration + HEARTBEAT_MERGE_GAP_SECS {
                // Extend existing event
                events::extend_event(conn, last.id, elapsed)?;
                return Ok(());
            }
        }
    }

    // Insert new event
    events::insert_event(
        conn,
        device_id,
        &now,
        0.0,
        &snapshot.app,
        &snapshot.title,
        snapshot.url.as_deref(),
        snapshot.url_domain.as_deref(),
        category_id,
        snapshot.is_afk,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::NamedTempFile;

    fn setup_db() -> (Connection, String) {
        let tmp = NamedTempFile::new().unwrap();
        let conn = db::open_db(tmp.path()).unwrap();
        db_categories::seed_builtin_categories(&conn).unwrap();

        // Create a test device
        let device_id = "test-device";
        conn.execute(
            "INSERT INTO devices (id, name, platform, last_sync) VALUES (?1, ?2, ?3, datetime('now'))",
            rusqlite::params![device_id, "test", "macos"],
        ).unwrap();

        (conn, device_id.to_string())
    }

    #[test]
    fn test_heartbeat_creates_new_event() {
        let (conn, device_id) = setup_db();
        let snapshot = WatcherSnapshot {
            app: "Code".to_string(),
            title: "main.rs".to_string(),
            url: None,
            url_domain: None,
            is_afk: false,
        };

        process_heartbeat(&conn, &device_id, &snapshot).unwrap();

        let last = events::get_last_event(&conn, &device_id).unwrap().unwrap();
        assert_eq!(last.app, "Code");
        assert_eq!(last.title, "main.rs");
    }

    #[test]
    fn test_heartbeat_extends_same_activity() {
        let (conn, device_id) = setup_db();
        let snapshot = WatcherSnapshot {
            app: "Code".to_string(),
            title: "main.rs".to_string(),
            url: None,
            url_domain: None,
            is_afk: false,
        };

        process_heartbeat(&conn, &device_id, &snapshot).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        process_heartbeat(&conn, &device_id, &snapshot).unwrap();

        // Should still be just one event
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE device_id = ?1",
                rusqlite::params![device_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let last = events::get_last_event(&conn, &device_id).unwrap().unwrap();
        assert!(last.duration > 0.0);
    }

    #[test]
    fn test_heartbeat_new_event_on_app_change() {
        let (conn, device_id) = setup_db();

        let s1 = WatcherSnapshot {
            app: "Code".to_string(),
            title: "main.rs".to_string(),
            url: None,
            url_domain: None,
            is_afk: false,
        };
        process_heartbeat(&conn, &device_id, &s1).unwrap();

        let s2 = WatcherSnapshot {
            app: "Firefox".to_string(),
            title: "Google".to_string(),
            url: Some("https://google.com".to_string()),
            url_domain: Some("google.com".to_string()),
            is_afk: false,
        };
        process_heartbeat(&conn, &device_id, &s2).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE device_id = ?1",
                rusqlite::params![device_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }
}
