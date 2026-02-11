use crate::db;
use crate::db::devices as db_devices;
use crate::error::Result;
use crate::output;

pub fn cmd_list(json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    let devices = db_devices::list_devices(&conn)?;

    if json {
        output::print_json(&devices);
    } else {
        if devices.is_empty() {
            println!("No devices registered");
        } else {
            println!("{:<40} {:<20} {:<10} {}", "ID", "Name", "Platform", "Last Sync");
            println!("{:-<90}", "");
            for d in &devices {
                println!("{:<40} {:<20} {:<10} {}", d.id, d.name, d.platform, d.last_sync.to_rfc3339());
            }
        }
    }

    Ok(())
}
