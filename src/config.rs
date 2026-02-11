use crate::error::{Result, TimelyError};
use std::fs;
use std::path::PathBuf;

pub const POLL_INTERVAL_SECS: u64 = 5;
pub const HEARTBEAT_MERGE_GAP_SECS: f64 = 65.0;
pub const DB_FILENAME: &str = "timely.db";
pub const PID_FILENAME: &str = "timely.pid";
pub const LAUNCHD_LABEL: &str = "com.timely.daemon";
pub const BUNDLE_IDENTIFIER: &str = "com.timely.app";
pub const SYNC_DEFAULT_INTERVAL_SECS: u64 = 300;

pub fn data_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| TimelyError::Config("Cannot determine home directory".into()))?
        .join(".timely");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join(DB_FILENAME))
}

pub fn pid_path() -> Result<PathBuf> {
    Ok(data_dir()?.join(PID_FILENAME))
}

pub fn launchd_plist_path() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| TimelyError::Config("Cannot determine home directory".into()))?
        .join("Library")
        .join("LaunchAgents");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir.join(format!("{}.plist", LAUNCHD_LABEL)))
}
