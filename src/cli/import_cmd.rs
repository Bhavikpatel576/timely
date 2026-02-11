use crate::db;
use crate::db::{devices, events};
use crate::error::Result;
use crate::types::Event;

pub fn cmd_import(file: &str) -> Result<()> {
    let content = std::fs::read_to_string(file)?;

    // Try parsing as JSON envelope first
    let imported: Vec<Event> = if let Ok(envelope) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(data) = envelope.get("data") {
            serde_json::from_value(data.clone())?
        } else {
            serde_json::from_str(&content)?
        }
    } else {
        serde_json::from_str(&content)?
    };

    let conn = db::open_default_db()?;
    let device = devices::get_or_create_device(&conn)?;

    let mut count = 0;
    for event in &imported {
        events::insert_event(
            &conn,
            &device.id,
            &event.timestamp,
            event.duration,
            &event.app,
            &event.title,
            event.url.as_deref(),
            event.url_domain.as_deref(),
            event.category_id,
            event.is_afk,
        )?;
        count += 1;
    }

    println!("Imported {} events", count);
    Ok(())
}
