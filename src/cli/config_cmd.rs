use crate::db;
use crate::db::config_store;
use crate::error::Result;
use crate::output;

pub fn cmd_set(key: &str, value: &str) -> Result<()> {
    let conn = db::open_default_db()?;
    config_store::set(&conn, key, value)?;
    println!("{} = {}", key, value);
    Ok(())
}

pub fn cmd_get(key: &str) -> Result<()> {
    let conn = db::open_default_db()?;
    match config_store::get(&conn, key)? {
        Some(value) => println!("{} = {}", key, value),
        None => println!("{} is not set", key),
    }
    Ok(())
}

pub fn cmd_list(json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    let items = config_store::list(&conn)?;

    if json {
        let map: serde_json::Map<String, serde_json::Value> = items
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        output::print_json(&map);
    } else {
        if items.is_empty() {
            println!("No configuration values set");
        } else {
            for (key, value) in &items {
                println!("{} = {}", key, value);
            }
        }
    }

    Ok(())
}
