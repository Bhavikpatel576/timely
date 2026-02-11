pub mod heartbeat;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::config::{POLL_INTERVAL_SECS, SYNC_DEFAULT_INTERVAL_SECS};
use crate::db;
use crate::db::categories as db_categories;
use crate::db::config_store;
use crate::db::devices;
use crate::error::Result;
use crate::sync;
use crate::watchers;

pub fn run_daemon() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));

    // Register signal handlers
    let r = running.clone();
    signal_hook::flag::register(signal_hook::consts::SIGTERM, r.clone())
        .map_err(|e| crate::error::TimelyError::Io(e))?;
    signal_hook::flag::register(signal_hook::consts::SIGINT, r)
        .map_err(|e| crate::error::TimelyError::Io(e))?;

    let conn = db::open_default_db()?;
    db_categories::seed_builtin_categories(&conn)?;
    let device = devices::get_or_create_device(&conn)?;

    eprintln!("timely daemon started (device: {}, pid: {})", device.name, std::process::id());

    // Write PID file
    let pid_path = crate::config::pid_path()?;
    std::fs::write(&pid_path, std::process::id().to_string())?;

    // Sync configuration
    let sync_enabled = config_store::get(&conn, "sync.enabled")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);
    let sync_interval: u64 = config_store::get(&conn, "sync.interval_secs")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(SYNC_DEFAULT_INTERVAL_SECS);

    if sync_enabled {
        eprintln!("sync enabled (interval: {}s)", sync_interval);
    }

    let mut sync_counter: u64 = 0;

    while running.load(Ordering::Relaxed) {
        match watchers::collect_snapshot() {
            Ok(snapshot) => {
                if let Err(e) = heartbeat::process_heartbeat(&conn, &device.id, &snapshot) {
                    eprintln!("heartbeat error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("watcher error: {}", e);
            }
        }

        // Sync tick
        sync_counter += POLL_INTERVAL_SECS;
        if sync_enabled && sync_counter >= sync_interval {
            sync_counter = 0;
            if let Err(e) = sync::client::push_events(&conn, &device) {
                eprintln!("sync error: {}", e);
            }
        }

        // Sleep in small increments to check running flag
        for _ in 0..(POLL_INTERVAL_SECS * 10) {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    // Cleanup PID file
    let _ = std::fs::remove_file(&pid_path);
    eprintln!("timely daemon stopped");

    Ok(())
}
