pub mod schema;
pub mod events;
pub mod categories;
pub mod config_store;
pub mod devices;
pub mod sync;

use rusqlite::Connection;
use crate::error::Result;
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    schema::run_migrations(&conn)?;
    Ok(conn)
}

pub fn open_default_db() -> Result<Connection> {
    let path = crate::config::db_path()?;
    open_db(&path)
}
