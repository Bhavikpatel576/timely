use chrono::Utc;
use rusqlite::Connection;
use uuid::Uuid;
use crate::error::Result;
use crate::platform::Platform;
use crate::types::Device;

pub fn get_or_create_device(conn: &Connection) -> Result<Device> {
    let hostname = hostname();
    let platform = Platform::current()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check if device exists by name+platform
    let mut stmt = conn.prepare(
        "SELECT id, name, platform, last_sync FROM devices WHERE name = ?1 AND platform = ?2",
    )?;
    let mut rows = stmt.query(rusqlite::params![hostname, platform])?;

    if let Some(row) = rows.next()? {
        let ts_str: String = row.get(3)?;
        let last_sync = chrono::DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let device = Device {
            id: row.get(0)?,
            name: row.get(1)?,
            platform: row.get(2)?,
            last_sync,
        };

        // Update last_sync
        conn.execute(
            "UPDATE devices SET last_sync = ?1 WHERE id = ?2",
            rusqlite::params![Utc::now().to_rfc3339(), device.id],
        )?;

        Ok(device)
    } else {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        conn.execute(
            "INSERT INTO devices (id, name, platform, last_sync) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, hostname, platform, now.to_rfc3339()],
        )?;
        Ok(Device {
            id,
            name: hostname,
            platform,
            last_sync: now,
        })
    }
}

pub fn list_devices(conn: &Connection) -> Result<Vec<Device>> {
    let mut stmt = conn.prepare("SELECT id, name, platform, last_sync FROM devices ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        let ts_str: String = row.get(3)?;
        let last_sync = chrono::DateTime::parse_from_rfc3339(&ts_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        Ok(Device {
            id: row.get(0)?,
            name: row.get(1)?,
            platform: row.get(2)?,
            last_sync,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| {
            gethostname().unwrap_or_else(|| "unknown".to_string())
        })
}

fn gethostname() -> Option<String> {
    let mut buf = vec![0u8; 256];
    let ret = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if ret == 0 {
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8(buf[..end].to_vec()).ok()
    } else {
        None
    }
}
