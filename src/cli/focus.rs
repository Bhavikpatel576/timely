use crate::db;
use crate::error::Result;
use crate::output;
use crate::query;
use crate::query::focus;

pub fn cmd_focus(from: &str, to: &str, json: bool) -> Result<()> {
    let from_dt = query::parse_time(from)?;
    let to_dt = query::parse_time(to)?;

    let conn = db::open_default_db()?;
    let result = focus::build_focus(&conn, &from_dt, &to_dt)?;

    if json {
        output::print_json(&result);
    } else {
        println!("Focus Analysis ({} to {})", from, to);
        println!(
            "Active: {} | Focus Score: {} | Switches: {} ({}/hr)",
            result.total_active_time,
            result.focus_score,
            result.context_switches,
            result.switches_per_hour
        );
        println!("{:-<60}", "");

        if !result.deep_work_blocks.is_empty() {
            println!(
                "Deep Work Blocks ({}, longest: {}m):",
                result.deep_work_blocks.len(),
                result.longest_focus_minutes
            );
            for block in &result.deep_work_blocks {
                println!(
                    "  {} — {} [{}, {}]",
                    block.start, block.duration_time, block.category, block.apps.join(", ")
                );
            }
        } else {
            println!("No deep work blocks detected (minimum 5 minutes).");
        }

        if !result.top_distractions.is_empty() {
            println!("\nTop Distractions:");
            for d in &result.top_distractions {
                println!("  {:<20} {} switches, {:.0}s total", d.app, d.switches_to, d.total_seconds);
            }
        }
    }

    Ok(())
}
