use crate::db;
use crate::db::events;
use crate::error::{Result, TimelyError};
use crate::output;
use crate::query;

pub fn cmd_export(format: &str, from: &str, to: &str) -> Result<()> {
    let from_dt = query::parse_time(from)?;
    let to_dt = query::parse_time(to)?;

    let conn = db::open_default_db()?;
    let event_list = events::query_events(&conn, &from_dt, &to_dt, None)?;

    if event_list.is_empty() {
        return Err(TimelyError::NoData);
    }

    match format {
        "csv" => {
            println!("timestamp,duration,app,title,url,url_domain,category,is_afk");
            for e in &event_list {
                println!(
                    "{},{},{},{},{},{},{},{}",
                    e.timestamp.to_rfc3339(),
                    e.duration,
                    csv_escape(&e.app),
                    csv_escape(&e.title),
                    csv_escape(e.url.as_deref().unwrap_or("")),
                    csv_escape(e.url_domain.as_deref().unwrap_or("")),
                    csv_escape(e.category_name.as_deref().unwrap_or("")),
                    e.is_afk,
                );
            }
        }
        _ => {
            // JSON
            output::print_json(&event_list);
        }
    }

    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
