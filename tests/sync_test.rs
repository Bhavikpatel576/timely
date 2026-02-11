use timely::db;
use timely::db::config_store;
use timely::db::events;
use timely::db::sync as db_sync;
use tempfile::NamedTempFile;

fn setup_db() -> rusqlite::Connection {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();

    // Insert a test device
    conn.execute(
        "INSERT INTO devices (id, name, platform, last_sync) VALUES (?1, ?2, ?3, datetime('now'))",
        rusqlite::params!["device-a", "macbook-pro", "macos"],
    )
    .unwrap();

    conn
}

fn insert_test_event(
    conn: &rusqlite::Connection,
    device_id: &str,
    timestamp: &str,
    duration: f64,
    app: &str,
    title: &str,
) {
    conn.execute(
        "INSERT INTO events (device_id, timestamp, duration, app, title, is_afk)
         VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        rusqlite::params![device_id, timestamp, duration, app, title],
    )
    .unwrap();
}

// --- Event dedup tests ---

#[test]
fn test_upsert_remote_event_inserts_new() {
    let conn = setup_db();

    let is_new = db_sync::upsert_remote_event(
        &conn,
        "device-a",
        "2025-01-15T10:00:00+00:00",
        30.0,
        "Code",
        "main.rs",
        None,
        None,
        None,
        false,
    )
    .unwrap();

    assert!(is_new, "Should insert new event");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_upsert_remote_event_dedup_same_event() {
    let conn = setup_db();

    // Insert first time
    let is_new1 = db_sync::upsert_remote_event(
        &conn,
        "device-a",
        "2025-01-15T10:00:00+00:00",
        30.0,
        "Code",
        "main.rs",
        None,
        None,
        None,
        false,
    )
    .unwrap();
    assert!(is_new1);

    // Insert same event again (same device, timestamp, app, title)
    let is_new2 = db_sync::upsert_remote_event(
        &conn,
        "device-a",
        "2025-01-15T10:00:00+00:00",
        30.0,
        "Code",
        "main.rs",
        None,
        None,
        None,
        false,
    )
    .unwrap();
    assert!(!is_new2, "Duplicate should not be new");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1, "Should not create duplicate");
}

#[test]
fn test_upsert_remote_event_takes_max_duration() {
    let conn = setup_db();

    // Insert with duration=30
    db_sync::upsert_remote_event(
        &conn,
        "device-a",
        "2025-01-15T10:00:00+00:00",
        30.0,
        "Code",
        "main.rs",
        None,
        None,
        None,
        false,
    )
    .unwrap();

    // Push same event with longer duration=60
    db_sync::upsert_remote_event(
        &conn,
        "device-a",
        "2025-01-15T10:00:00+00:00",
        60.0,
        "Code",
        "main.rs",
        None,
        None,
        None,
        false,
    )
    .unwrap();

    let duration: f64 = conn
        .query_row(
            "SELECT duration FROM events WHERE device_id = 'device-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(duration, 60.0, "Should take MAX(duration)");

    // Push with shorter duration=20 — should NOT downgrade
    db_sync::upsert_remote_event(
        &conn,
        "device-a",
        "2025-01-15T10:00:00+00:00",
        20.0,
        "Code",
        "main.rs",
        None,
        None,
        None,
        false,
    )
    .unwrap();

    let duration2: f64 = conn
        .query_row(
            "SELECT duration FROM events WHERE device_id = 'device-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(duration2, 60.0, "Should not downgrade duration");
}

// --- Device registration tests ---

#[test]
fn test_upsert_remote_device_insert() {
    let conn = setup_db();

    db_sync::upsert_remote_device(&conn, "device-b", "macbook-air", "macos").unwrap();

    let name: String = conn
        .query_row(
            "SELECT name FROM devices WHERE id = 'device-b'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "macbook-air");
}

#[test]
fn test_upsert_remote_device_updates_existing() {
    let conn = setup_db();

    // device-a already exists from setup_db
    db_sync::upsert_remote_device(&conn, "device-a", "macbook-pro-new", "macos").unwrap();

    let name: String = conn
        .query_row(
            "SELECT name FROM devices WHERE id = 'device-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "macbook-pro-new");

    // Should still be exactly 1 device with that id
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM devices WHERE id = 'device-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

// --- Sync log tests ---

#[test]
fn test_sync_log_create_and_update() {
    let conn = setup_db();

    // Initially no sync log
    let log = db_sync::get_sync_log(&conn, "device-a").unwrap();
    assert!(log.is_none());

    // Create sync log
    db_sync::update_sync_log(&conn, "device-a", 42).unwrap();
    let log = db_sync::get_sync_log(&conn, "device-a").unwrap().unwrap();
    assert_eq!(log.0, 42);

    // Update sync log
    db_sync::update_sync_log(&conn, "device-a", 100).unwrap();
    let log = db_sync::get_sync_log(&conn, "device-a").unwrap().unwrap();
    assert_eq!(log.0, 100);
}

// --- query_events_after_id tests ---

#[test]
fn test_query_events_after_id() {
    let conn = setup_db();

    insert_test_event(&conn, "device-a", "2025-01-15T10:00:00+00:00", 5.0, "Code", "file1.rs");
    insert_test_event(&conn, "device-a", "2025-01-15T10:01:00+00:00", 5.0, "Code", "file2.rs");
    insert_test_event(&conn, "device-a", "2025-01-15T10:02:00+00:00", 5.0, "Safari", "Google");

    // Get all events after id 0
    let all = events::query_events_after_id(&conn, "device-a", 0, 100).unwrap();
    assert_eq!(all.len(), 3);

    // Get events after the first one
    let first_id = all[0].id;
    let after_first = events::query_events_after_id(&conn, "device-a", first_id, 100).unwrap();
    assert_eq!(after_first.len(), 2);
    assert_eq!(after_first[0].title, "file2.rs");

    // Limit works
    let limited = events::query_events_after_id(&conn, "device-a", 0, 1).unwrap();
    assert_eq!(limited.len(), 1);
}

// --- Device event counts ---

#[test]
fn test_device_event_counts() {
    let conn = setup_db();

    // Add a second device
    conn.execute(
        "INSERT INTO devices (id, name, platform, last_sync) VALUES (?1, ?2, ?3, datetime('now'))",
        rusqlite::params!["device-b", "macbook-air", "macos"],
    )
    .unwrap();

    insert_test_event(&conn, "device-a", "2025-01-15T10:00:00+00:00", 5.0, "Code", "file1.rs");
    insert_test_event(&conn, "device-a", "2025-01-15T10:01:00+00:00", 5.0, "Code", "file2.rs");
    insert_test_event(&conn, "device-b", "2025-01-15T10:00:00+00:00", 5.0, "Safari", "Google");

    let counts = db_sync::get_device_event_counts(&conn).unwrap();
    assert_eq!(counts.len(), 2);

    // Find device-a
    let a = counts.iter().find(|(id, ..)| id == "device-a").unwrap();
    assert_eq!(a.4, 2); // 2 events

    let b = counts.iter().find(|(id, ..)| id == "device-b").unwrap();
    assert_eq!(b.4, 1); // 1 event

    let total = db_sync::get_total_event_count(&conn).unwrap();
    assert_eq!(total, 3);
}

// --- Config-based sync settings ---

#[test]
fn test_sync_config_roundtrip() {
    let conn = setup_db();

    config_store::set(&conn, "sync.hub_url", "http://192.168.1.10:8080").unwrap();
    config_store::set(&conn, "sync.api_key", "test-secret-key").unwrap();
    config_store::set(&conn, "sync.enabled", "true").unwrap();

    assert_eq!(
        config_store::get(&conn, "sync.hub_url").unwrap().unwrap(),
        "http://192.168.1.10:8080"
    );
    assert_eq!(
        config_store::get(&conn, "sync.api_key").unwrap().unwrap(),
        "test-secret-key"
    );
    assert_eq!(
        config_store::get(&conn, "sync.enabled").unwrap().unwrap(),
        "true"
    );
}

// --- Full push flow (DB-level, no HTTP) ---

#[test]
fn test_full_sync_push_flow_db_level() {
    let conn = setup_db();

    // Simulate events created by local daemon
    insert_test_event(&conn, "device-a", "2025-01-15T10:00:00+00:00", 5.0, "Code", "file1.rs");
    insert_test_event(&conn, "device-a", "2025-01-15T10:01:00+00:00", 10.0, "Code", "file2.rs");

    // Get events after id=0 (what the sync client would do)
    let events_to_push = events::query_events_after_id(&conn, "device-a", 0, 1000).unwrap();
    assert_eq!(events_to_push.len(), 2);

    // Simulate hub receiving these events (into same DB for test)
    let mut accepted = 0;
    let mut duplicates = 0;
    for event in &events_to_push {
        let is_new = db_sync::upsert_remote_event(
            &conn,
            &event.device_id,
            &event.timestamp.to_rfc3339(),
            event.duration,
            &event.app,
            &event.title,
            event.url.as_deref(),
            event.url_domain.as_deref(),
            event.category_id,
            event.is_afk,
        )
        .unwrap();

        if is_new {
            accepted += 1;
        } else {
            duplicates += 1;
        }
    }

    // All should be duplicates since they're already in the same DB
    assert_eq!(accepted, 0);
    assert_eq!(duplicates, 2);

    // Update sync log
    let last_id = events_to_push.last().unwrap().id;
    db_sync::update_sync_log(&conn, "device-a", last_id).unwrap();

    let log = db_sync::get_sync_log(&conn, "device-a").unwrap().unwrap();
    assert_eq!(log.0, last_id);

    // No more events to push
    let remaining = events::query_events_after_id(&conn, "device-a", last_id, 1000).unwrap();
    assert_eq!(remaining.len(), 0);
}
