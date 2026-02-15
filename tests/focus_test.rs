use chrono::{Duration, Utc};
use timely::db;
use timely::db::categories as db_categories;
use timely::db::events;
use timely::query::focus;
use tempfile::NamedTempFile;

fn setup_focus_db() -> rusqlite::Connection {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();
    db_categories::seed_builtin_categories(&conn).unwrap();

    let device_id = "test-device";
    conn.execute(
        "INSERT INTO devices (id, name, platform, last_sync) VALUES (?1, ?2, ?3, datetime('now'))",
        rusqlite::params![device_id, "test", "macos"],
    ).unwrap();

    let now = Utc::now();
    let coding_cat = db_categories::get_category_by_name(&conn, "work/coding").unwrap().unwrap();
    let chat_cat = db_categories::get_category_by_name(&conn, "communication/chat").unwrap().unwrap();
    let video_cat = db_categories::get_category_by_name(&conn, "entertainment/video").unwrap().unwrap();

    // Deep work block: 3 consecutive coding events (total 1500s = 25 min)
    events::insert_event(&conn, device_id, &(now - Duration::seconds(3600)), 600.0,
        "Code", "main.rs", None, None, Some(coding_cat.id), false).unwrap();
    events::insert_event(&conn, device_id, &(now - Duration::seconds(3000)), 500.0,
        "Terminal", "cargo build", None, None, Some(coding_cat.id), false).unwrap();
    events::insert_event(&conn, device_id, &(now - Duration::seconds(2500)), 400.0,
        "Code", "lib.rs", None, None, Some(coding_cat.id), false).unwrap();

    // Distraction: switch to Slack (non-productive)
    events::insert_event(&conn, device_id, &(now - Duration::seconds(2100)), 300.0,
        "Slack", "#general", None, None, Some(chat_cat.id), false).unwrap();

    // Back to coding
    events::insert_event(&conn, device_id, &(now - Duration::seconds(1800)), 600.0,
        "Code", "test.rs", None, None, Some(coding_cat.id), false).unwrap();

    // Video distraction
    events::insert_event(&conn, device_id, &(now - Duration::seconds(1200)), 200.0,
        "Safari", "YouTube", Some("https://youtube.com"), Some("youtube.com"),
        Some(video_cat.id), false).unwrap();

    // More coding
    events::insert_event(&conn, device_id, &(now - Duration::seconds(1000)), 500.0,
        "Code", "mod.rs", None, None, Some(coding_cat.id), false).unwrap();

    // AFK event (should be excluded)
    events::insert_event(&conn, device_id, &(now - Duration::seconds(500)), 300.0,
        "loginwindow", "Locked", None, None, None, true).unwrap();

    conn
}

#[test]
fn test_focus_basic() {
    let conn = setup_focus_db();
    let from = Utc::now() - Duration::hours(2);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to).unwrap();

    // Total active should exclude AFK (300s)
    assert!(result.total_active_seconds > 0.0);
    // AFK duration (300s) should not be counted
    assert!((result.total_active_seconds - 3100.0).abs() < 1.0,
        "Expected ~3100s active, got {}", result.total_active_seconds);
}

#[test]
fn test_focus_context_switches() {
    let conn = setup_focus_db();
    let from = Utc::now() - Duration::hours(2);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to).unwrap();

    // Switches: coding→chat, chat→coding, coding→video, video→coding = 4
    assert_eq!(result.context_switches, 4,
        "Expected 4 context switches, got {}", result.context_switches);
}

#[test]
fn test_focus_deep_work_blocks() {
    let conn = setup_focus_db();
    let from = Utc::now() - Duration::hours(2);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to).unwrap();

    // First block: 3 coding events = 1500s (≥300s, qualifies)
    // Second block: coding 600s (≥300s, qualifies)
    // Third block: coding 500s (≥300s, qualifies)
    assert!(!result.deep_work_blocks.is_empty(),
        "Should have at least one deep work block");

    // All deep work blocks should be in coding category
    for block in &result.deep_work_blocks {
        assert_eq!(block.category, "work/coding");
        assert!(block.duration_seconds >= 300.0);
    }
}

#[test]
fn test_focus_distractions() {
    let conn = setup_focus_db();
    let from = Utc::now() - Duration::hours(2);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to).unwrap();

    // Slack and Safari should appear as distractions (switched from productive→non-productive)
    let distraction_apps: Vec<&str> = result.top_distractions.iter().map(|d| d.app.as_str()).collect();
    assert!(distraction_apps.contains(&"Slack"), "Slack should be a distraction");
    assert!(distraction_apps.contains(&"Safari"), "Safari should be a distraction");
    assert!(result.top_distractions.len() <= 5);
}

#[test]
fn test_focus_score_range() {
    let conn = setup_focus_db();
    let from = Utc::now() - Duration::hours(2);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to).unwrap();

    assert!(result.focus_score <= 100, "Focus score should be ≤ 100");
    // With substantial deep work, score should be > 0
    assert!(result.focus_score > 0, "Focus score should be > 0 with deep work");
}

#[test]
fn test_focus_empty_data() {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();

    let from = Utc::now() - Duration::hours(1);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to);
    assert!(result.is_err(), "Should return error on empty data");
}

#[test]
fn test_focus_longest_block() {
    let conn = setup_focus_db();
    let from = Utc::now() - Duration::hours(2);
    let to = Utc::now();

    let result = focus::build_focus(&conn, &from, &to).unwrap();

    assert!(result.longest_focus_minutes > 0.0,
        "Longest focus should be > 0 with deep work blocks");
    // Longest block is the first one: 1500s = 25 min
    assert!(result.longest_focus_minutes >= 5.0,
        "Longest focus should be ≥ 5 minutes, got {}", result.longest_focus_minutes);
}
