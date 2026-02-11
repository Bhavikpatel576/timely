use timely::categories;
use timely::db;
use timely::db::categories as db_categories;
use timely::types::WatcherSnapshot;
use tempfile::NamedTempFile;

fn setup_db() -> rusqlite::Connection {
    let tmp = NamedTempFile::new().unwrap();
    let conn = db::open_db(tmp.path()).unwrap();
    db_categories::seed_builtin_categories(&conn).unwrap();
    conn
}

#[test]
fn test_classify_ide() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Code".to_string(),
        title: "main.rs".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some());

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "work/coding");
    assert_eq!(cat.productivity_score, 2.0);
}

#[test]
fn test_classify_terminal() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Terminal".to_string(),
        title: "bash".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some());

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "work/terminal");
}

#[test]
fn test_classify_browser_youtube() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Safari".to_string(),
        title: "YouTube".to_string(),
        url: Some("https://www.youtube.com/watch?v=123".to_string()),
        url_domain: Some("www.youtube.com".to_string()),
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some());

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "entertainment/video");
}

#[test]
fn test_classify_browser_github() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Google Chrome".to_string(),
        title: "Pull Request #42".to_string(),
        url: Some("https://github.com/user/repo/pull/42".to_string()),
        url_domain: Some("github.com".to_string()),
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some());

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "reference/docs");
}

#[test]
fn test_classify_unknown_app_returns_none() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "SomeRandomApp".to_string(),
        title: "Unknown Window".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_none(), "Unknown apps should return None (uncategorized)");
}

#[test]
fn test_user_rule_overrides_builtin() {
    let conn = setup_db();

    // Add a user rule with higher priority
    let custom_cat_id = db_categories::insert_category(&conn, "work/review", None, 1.5).unwrap();
    db_categories::insert_rule(&conn, custom_cat_id, "url_domain", "github.com", false, 200).unwrap();

    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Google Chrome".to_string(),
        title: "PR Review".to_string(),
        url: Some("https://github.com/org/repo/pull/1".to_string()),
        url_domain: Some("github.com".to_string()),
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules).unwrap();
    let cat = db_categories::get_category_by_id(&conn, cat_id).unwrap().unwrap();
    assert_eq!(cat.name, "work/review", "User rule should override builtin");
}

#[test]
fn test_classify_claude_code() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    // Simulates what happens when TUI detector sees "claude" running in terminal
    // and replaces app name with "Claude Code"
    let snapshot = WatcherSnapshot {
        app: "Claude Code".to_string(),
        title: "bhavikpatel@Mac:~/projects".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some(), "Claude Code should be classified");

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "work/ai-tools");
    assert_eq!(cat.productivity_score, 2.0);
}

#[test]
fn test_classify_codex_cli() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Codex CLI".to_string(),
        title: "zsh".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some(), "Codex CLI should be classified");

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "work/ai-tools");
}

#[test]
fn test_classify_aider() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "Aider".to_string(),
        title: "aider".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some(), "Aider should be classified");

    let cat = db_categories::get_category_by_id(&conn, cat_id.unwrap()).unwrap().unwrap();
    assert_eq!(cat.name, "work/ai-tools");
}

#[test]
fn test_case_insensitive_matching() {
    let conn = setup_db();
    let rules = db_categories::list_rules(&conn).unwrap();

    let snapshot = WatcherSnapshot {
        app: "code".to_string(), // lowercase
        title: "test.rs".to_string(),
        url: None,
        url_domain: None,
        is_afk: false,
    };

    let cat_id = categories::classify(&snapshot, &rules);
    assert!(cat_id.is_some(), "Matching should be case-insensitive");
}
