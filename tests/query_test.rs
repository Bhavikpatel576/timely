use chrono::{Duration, Utc};
use timely::db;
use timely::db::categories as db_categories;
use timely::db::events;
use timely::query::{self, summary, timeline};
use timely::query::summary::GroupBy;
use tempfile::NamedTempFile;

fn setup_db_with_events() -> rusqlite::Connection {
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

    // Insert test events spanning different times and categories
    events::insert_event(&conn, device_id, &(now - Duration::hours(3)), 3600.0,
        "Code", "main.rs", None, None, Some(coding_cat.id), false).unwrap();
    events::insert_event(&conn, device_id, &(now - Duration::hours(2)), 1800.0,
        "Slack", "#dev", None, None, Some(chat_cat.id), false).unwrap();
    events::insert_event(&conn, device_id, &(now - Duration::minutes(90)), 900.0,
        "Safari", "YouTube", Some("https://youtube.com"), Some("youtube.com"),
        Some(video_cat.id), false).unwrap();
    events::insert_event(&conn, device_id, &(now - Duration::hours(1)), 1200.0,
        "Code", "lib.rs", None, None, Some(coding_cat.id), false).unwrap();

    conn
}

#[test]
fn test_parse_time_now() {
    let result = query::parse_time("now").unwrap();
    let diff = (Utc::now() - result).num_seconds().abs();
    assert!(diff < 2, "now should be within 2 seconds of current time");
}

#[test]
fn test_parse_time_today() {
    let result = query::parse_time("today").unwrap();
    assert!(result < Utc::now());
    assert!((Utc::now() - result).num_hours() < 24);
}

#[test]
fn test_parse_time_relative_days() {
    let result = query::parse_time("7d").unwrap();
    let expected = Utc::now() - Duration::days(7);
    let diff = (result - expected).num_seconds().abs();
    assert!(diff < 2);
}

#[test]
fn test_parse_time_relative_hours() {
    let result = query::parse_time("2h").unwrap();
    let expected = Utc::now() - Duration::hours(2);
    let diff = (result - expected).num_seconds().abs();
    assert!(diff < 2);
}

#[test]
fn test_parse_time_relative_minutes() {
    let result = query::parse_time("30m").unwrap();
    let expected = Utc::now() - Duration::minutes(30);
    let diff = (result - expected).num_seconds().abs();
    assert!(diff < 2);
}

#[test]
fn test_parse_time_iso_date() {
    let result = query::parse_time("2025-03-15").unwrap();
    assert_eq!(result.date_naive().to_string(), "2025-03-15");
}

#[test]
fn test_parse_time_invalid() {
    let result = query::parse_time("garbage");
    assert!(result.is_err());
}

#[test]
fn test_summary_by_category() {
    let conn = setup_db_with_events();
    let from = Utc::now() - Duration::hours(4);
    let to = Utc::now();

    let result = summary::build_summary(&conn, &from, &to, GroupBy::Category).unwrap();

    assert!(result.total_seconds > 0.0);
    assert!(!result.groups.is_empty());

    // work/coding should be the largest group (3600 + 1200 = 4800s)
    let coding_group = result.groups.iter().find(|g| g.label == "work/coding");
    assert!(coding_group.is_some());
    assert_eq!(coding_group.unwrap().seconds, 4800.0);

    // Productivity should be positive (dominated by coding)
    assert!(result.productivity_score > 0.0);
}

#[test]
fn test_summary_by_app() {
    let conn = setup_db_with_events();
    let from = Utc::now() - Duration::hours(4);
    let to = Utc::now();

    let result = summary::build_summary(&conn, &from, &to, GroupBy::App).unwrap();

    let code_group = result.groups.iter().find(|g| g.label == "Code");
    assert!(code_group.is_some());

    let slack_group = result.groups.iter().find(|g| g.label == "Slack");
    assert!(slack_group.is_some());
}

#[test]
fn test_summary_percentages_sum_to_100() {
    let conn = setup_db_with_events();
    let from = Utc::now() - Duration::hours(4);
    let to = Utc::now();

    let result = summary::build_summary(&conn, &from, &to, GroupBy::Category).unwrap();

    let total_pct: f64 = result.groups.iter().map(|g| g.percentage).sum();
    assert!((total_pct - 100.0).abs() < 1.0, "Percentages should sum to ~100%");
}

#[test]
fn test_summary_no_data() {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();

    let from = Utc::now() - Duration::hours(1);
    let to = Utc::now();

    let result = summary::build_summary(&conn, &from, &to, GroupBy::Category);
    assert!(result.is_err());
}

#[test]
fn test_timeline() {
    let conn = setup_db_with_events();
    let from = Utc::now() - Duration::hours(4);
    let to = Utc::now();

    let result = timeline::build_timeline(&conn, &from, &to, None).unwrap();

    assert_eq!(result.count, 4);
    // Should be in chronological order
    for i in 1..result.entries.len() {
        assert!(result.entries[i].timestamp >= result.entries[i - 1].timestamp);
    }
}

#[test]
fn test_timeline_with_limit() {
    let conn = setup_db_with_events();
    let from = Utc::now() - Duration::hours(4);
    let to = Utc::now();

    let result = timeline::build_timeline(&conn, &from, &to, Some(2)).unwrap();
    assert_eq!(result.count, 2);
}

#[test]
fn test_timeline_no_data() {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();

    let from = Utc::now() - Duration::hours(1);
    let to = Utc::now();

    let result = timeline::build_timeline(&conn, &from, &to, None);
    assert!(result.is_err());
}
