use timely::db;
use timely::db::categories as db_categories;
use timely::db::events;
use timely::daemon::heartbeat;
use timely::types::WatcherSnapshot;
use tempfile::NamedTempFile;

fn setup_db() -> (rusqlite::Connection, String) {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();
    db_categories::seed_builtin_categories(&conn).unwrap();

    let device_id = "test-device";
    conn.execute(
        "INSERT INTO devices (id, name, platform, last_sync) VALUES (?1, ?2, ?3, datetime('now'))",
        rusqlite::params![device_id, "test", "macos"],
    ).unwrap();

    (conn, device_id.to_string())
}

#[test]
fn test_first_heartbeat_creates_event() {
    let (conn, device_id) = setup_db();
    let snapshot = WatcherSnapshot {
        app: "Code".to_string(),
        title: "main.rs — timely".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    heartbeat::process_heartbeat(&conn, &device_id, &snapshot).unwrap();

    let last = events::get_last_event(&conn, &device_id).unwrap().unwrap();
    assert_eq!(last.app, "Code");
    assert_eq!(last.title, "main.rs — timely");
    assert!(!last.is_afk);
    // Should be classified as work/coding
    assert!(last.category_name.is_some());
    assert_eq!(last.category_name.unwrap(), "work/coding");
}

#[test]
fn test_heartbeat_merges_same_activity() {
    let (conn, device_id) = setup_db();
    let snapshot = WatcherSnapshot {
        app: "Slack".to_string(),
        title: "#general".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    heartbeat::process_heartbeat(&conn, &device_id, &snapshot).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    heartbeat::process_heartbeat(&conn, &device_id, &snapshot).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    heartbeat::process_heartbeat(&conn, &device_id, &snapshot).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE device_id = ?1",
            rusqlite::params![device_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 1, "Should merge into single event");

    let last = events::get_last_event(&conn, &device_id).unwrap().unwrap();
    assert!(last.duration > 0.0, "Duration should have been extended");
}

#[test]
fn test_heartbeat_splits_on_app_change() {
    let (conn, device_id) = setup_db();

    heartbeat::process_heartbeat(
        &conn,
        &device_id,
        &WatcherSnapshot {
            app: "Code".to_string(),
            title: "lib.rs".to_string(),
            url: None,
            url_domain: None,
            is_afk: false,
        },
    ).unwrap();

    heartbeat::process_heartbeat(
        &conn,
        &device_id,
        &WatcherSnapshot {
            app: "Safari".to_string(),
            title: "Rust docs".to_string(),
            url: Some("https://doc.rust-lang.org".to_string()),
            url_domain: Some("doc.rust-lang.org".to_string()),
            is_afk: false,
        },
    ).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE device_id = ?1",
            rusqlite::params![device_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 2, "Different apps should create separate events");
}

#[test]
fn test_heartbeat_splits_on_afk_change() {
    let (conn, device_id) = setup_db();

    heartbeat::process_heartbeat(
        &conn,
        &device_id,
        &WatcherSnapshot {
            app: "Code".to_string(),
            title: "main.rs".to_string(),
            url: None,
            url_domain: None,
            is_afk: false,
        },
    ).unwrap();

    heartbeat::process_heartbeat(
        &conn,
        &device_id,
        &WatcherSnapshot {
            app: "Code".to_string(),
            title: "main.rs".to_string(),
            url: None,
            url_domain: None,
            is_afk: true,
        },
    ).unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM events WHERE device_id = ?1",
            rusqlite::params![device_id],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(count, 2, "AFK change should create separate event");
}
