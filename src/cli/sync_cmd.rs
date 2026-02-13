use crate::db;
use crate::db::config_store;
use crate::db::devices;
use crate::db::sync as db_sync;
use crate::error::Result;
use crate::output;
use crate::sync::client;

pub fn cmd_setup(hub: &str, key: Option<&str>, json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    let device = devices::get_or_create_device(&conn)?;

    // Store sync config
    config_store::set(&conn, "sync.hub_url", hub)?;
    if let Some(k) = key {
        config_store::set(&conn, "sync.api_key", k)?;
    }
    config_store::set(&conn, "sync.enabled", "true")?;

    // Register with hub
    eprintln!("Registering device '{}' with hub at {}...", device.name, hub);
    client::register_with_hub(&conn, &device)?;

    if json {
        output::print_json(&serde_json::json!({
            "hub_url": hub,
            "device_id": device.id,
            "device_name": device.name,
            "registered": true,
            "sync_enabled": true,
            "auth": key.is_some(),
        }));
    } else {
        println!("Sync configured");
        println!("  Hub:    {}", hub);
        println!("  Device: {} ({})", device.name, device.id);
        println!("  Auth:   {}", if key.is_some() { "yes" } else { "no" });
    }

    Ok(())
}

pub fn cmd_push(json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    let device = devices::get_or_create_device(&conn)?;

    eprintln!("Pushing unsynced events to hub...");
    let result = client::push_events(&conn, &device)?;

    if json {
        output::print_json(&serde_json::json!({
            "accepted": result.total_accepted,
            "duplicates": result.total_duplicates,
            "batches": result.batches,
        }));
    } else {
        println!("Push complete: {} accepted, {} duplicates ({} batches)",
            result.total_accepted, result.total_duplicates, result.batches);
    }

    Ok(())
}

pub fn cmd_status(json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    let device = devices::get_or_create_device(&conn)?;

    let hub_url = config_store::get(&conn, "sync.hub_url")?;
    let api_key = config_store::get(&conn, "sync.api_key")?;
    let sync_enabled = config_store::get(&conn, "sync.enabled")?
        .map(|v| v == "true")
        .unwrap_or(false);

    // Local sync info
    let sync_log = db_sync::get_sync_log(&conn, &device.id)?;
    let last_synced_id = sync_log.as_ref().map(|(id, _)| *id).unwrap_or(0);
    let last_sync_at = sync_log.map(|(_, ts)| ts);

    // Count pending events
    let pending: i64 = conn.query_row(
        "SELECT COUNT(*) FROM events WHERE device_id = ?1 AND id > ?2",
        rusqlite::params![device.id, last_synced_id],
        |row| row.get(0),
    )?;

    // Try to reach hub for remote status
    let hub_reachable;
    let remote_devices;
    if let Some(ref url) = hub_url {
        match client::get_hub_status(url, &api_key) {
            Ok(status) => {
                hub_reachable = true;
                remote_devices = status.get("data").cloned();
            }
            Err(_) => {
                hub_reachable = false;
                remote_devices = None;
            }
        }
    } else {
        hub_reachable = false;
        remote_devices = None;
    }

    let response = serde_json::json!({
        "sync_enabled": sync_enabled,
        "hub_url": hub_url,
        "hub_reachable": hub_reachable,
        "last_sync_at": last_sync_at,
        "pending_events": pending,
        "remote": remote_devices,
    });

    if json {
        output::print_json(&response);
    } else {
        println!("Sync Status");
        println!("{:-<40}", "");
        println!("Enabled:    {}", sync_enabled);
        println!("Hub URL:    {}", hub_url.as_deref().unwrap_or("(not set)"));
        println!("Reachable:  {}", hub_reachable);
        println!(
            "Last sync:  {}",
            last_sync_at.as_deref().unwrap_or("never")
        );
        println!("Pending:    {} events", pending);

        if let Some(ref data) = remote_devices {
            if let Some(devices) = data.get("devices").and_then(|d| d.as_array()) {
                println!("\nRegistered Devices:");
                for d in devices {
                    println!(
                        "  {} ({}) — {} events, last sync: {}",
                        d["name"].as_str().unwrap_or("?"),
                        d["platform"].as_str().unwrap_or("?"),
                        d["event_count"].as_i64().unwrap_or(0),
                        d["last_sync"].as_str().unwrap_or("?"),
                    );
                }
            }
        }
    }

    Ok(())
}
