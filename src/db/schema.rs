use rusqlite::Connection;
use crate::error::Result;

const MIGRATIONS: &[&str] = &[
    // Version 1: Initial schema
    "CREATE TABLE IF NOT EXISTS devices (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        platform TEXT NOT NULL,
        last_sync TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS categories (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        parent_id INTEGER REFERENCES categories(id),
        productivity_score REAL NOT NULL DEFAULT 0.0
    );

    CREATE TABLE IF NOT EXISTS category_rules (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        category_id INTEGER NOT NULL REFERENCES categories(id),
        field TEXT NOT NULL CHECK(field IN ('app', 'title', 'url_domain')),
        pattern TEXT NOT NULL,
        is_builtin INTEGER NOT NULL DEFAULT 0,
        priority INTEGER NOT NULL DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        device_id TEXT NOT NULL REFERENCES devices(id),
        timestamp TEXT NOT NULL,
        duration REAL NOT NULL DEFAULT 0.0,
        app TEXT NOT NULL DEFAULT '',
        title TEXT NOT NULL DEFAULT '',
        url TEXT,
        url_domain TEXT,
        category_id INTEGER REFERENCES categories(id),
        is_afk INTEGER NOT NULL DEFAULT 0
    );

    CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
    CREATE INDEX IF NOT EXISTS idx_events_device ON events(device_id);
    CREATE INDEX IF NOT EXISTS idx_events_category ON events(category_id);
    CREATE INDEX IF NOT EXISTS idx_category_rules_field ON category_rules(field, pattern);

    CREATE TABLE IF NOT EXISTS config (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );",
    // Version 2: Sync log for multi-device sync
    "CREATE TABLE IF NOT EXISTS sync_log (
        device_id TEXT PRIMARY KEY,
        last_synced_event_id INTEGER NOT NULL DEFAULT 0,
        last_sync_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i32;
        if version > current_version {
            conn.execute_batch(migration)?;
            conn.pragma_update(None, "user_version", version)?;
        }
    }

    Ok(())
}
