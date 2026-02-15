use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use crate::error::{Result, TimelyError};
use crate::types::{DeepWorkBlock, DistractionEntry, FocusResponse, format_duration};

struct FocusEvent {
    timestamp: String,
    duration: f64,
    app: String,
    category: String,
    productivity_score: f64,
    is_afk: bool,
}

pub fn build_focus(
    conn: &Connection,
    from: &DateTime<Utc>,
    to: &DateTime<Utc>,
) -> Result<FocusResponse> {
    let from_str = from.to_rfc3339();
    let to_str = to.to_rfc3339();

    let mut stmt = conn.prepare(
        "SELECT e.timestamp, e.duration, e.app,
                COALESCE(c.name, 'uncategorized') as category,
                COALESCE(c.productivity_score, 0.0) as prod_score,
                e.is_afk
         FROM events e
         LEFT JOIN categories c ON e.category_id = c.id
         WHERE e.timestamp >= ?1 AND e.timestamp <= ?2
         ORDER BY e.timestamp ASC"
    )?;

    let events: Vec<FocusEvent> = stmt.query_map(
        rusqlite::params![&from_str, &to_str],
        |row| {
            Ok(FocusEvent {
                timestamp: row.get(0)?,
                duration: row.get(1)?,
                app: row.get(2)?,
                category: row.get(3)?,
                productivity_score: row.get(4)?,
                is_afk: row.get::<_, i32>(5)? != 0,
            })
        },
    )?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    if events.is_empty() {
        return Err(TimelyError::NoData);
    }

    // Filter out AFK, compute total active time
    let active_events: Vec<&FocusEvent> = events.iter().filter(|e| !e.is_afk).collect();
    let total_active_seconds: f64 = active_events.iter().map(|e| e.duration).sum();

    if active_events.is_empty() {
        return Err(TimelyError::NoData);
    }

    // Count context switches (category changes between consecutive active events)
    let mut context_switches: u32 = 0;
    for i in 1..active_events.len() {
        if active_events[i].category != active_events[i - 1].category {
            context_switches += 1;
        }
    }

    let active_hours = total_active_seconds / 3600.0;
    let switches_per_hour = if active_hours > 0.0 {
        (context_switches as f64 / active_hours * 10.0).round() / 10.0
    } else {
        0.0
    };

    // Build deep work blocks: consecutive productive events in same category
    // Allow different apps within same category. Break on: category change,
    // non-productive event, or gap > 65s. Minimum block duration: 300s
    let mut deep_work_blocks: Vec<DeepWorkBlock> = Vec::new();
    let mut block_start: Option<usize> = None;
    let mut block_apps: Vec<String> = Vec::new();
    let mut block_duration: f64 = 0.0;

    for (i, event) in active_events.iter().enumerate() {
        let is_productive = event.productivity_score > 0.0;

        if let Some(start_idx) = block_start {
            let prev = active_events[i - 1];
            let same_category = event.category == active_events[start_idx].category;

            // Check gap: parse timestamps to detect > 65s gap
            let gap_ok = check_gap(&prev.timestamp, prev.duration, &event.timestamp);

            if is_productive && same_category && gap_ok {
                // Continue block
                block_duration += event.duration;
                if !block_apps.contains(&event.app) {
                    block_apps.push(event.app.clone());
                }
            } else {
                // End current block
                if block_duration >= 300.0 {
                    deep_work_blocks.push(DeepWorkBlock {
                        start: active_events[start_idx].timestamp.clone(),
                        end: prev.timestamp.clone(),
                        duration_seconds: block_duration,
                        duration_time: format_duration(block_duration),
                        category: active_events[start_idx].category.clone(),
                        apps: block_apps.clone(),
                    });
                }
                // Maybe start new block
                if is_productive {
                    block_start = Some(i);
                    block_apps = vec![event.app.clone()];
                    block_duration = event.duration;
                } else {
                    block_start = None;
                    block_apps.clear();
                    block_duration = 0.0;
                }
            }
        } else if is_productive {
            // Start new block
            block_start = Some(i);
            block_apps = vec![event.app.clone()];
            block_duration = event.duration;
        }
    }

    // Flush last block
    if let Some(start_idx) = block_start {
        if block_duration >= 300.0 {
            let last = active_events.last().unwrap();
            deep_work_blocks.push(DeepWorkBlock {
                start: active_events[start_idx].timestamp.clone(),
                end: last.timestamp.clone(),
                duration_seconds: block_duration,
                duration_time: format_duration(block_duration),
                category: active_events[start_idx].category.clone(),
                apps: block_apps,
            });
        }
    }

    let longest_focus_minutes = deep_work_blocks
        .iter()
        .map(|b| b.duration_seconds)
        .fold(0.0_f64, f64::max)
        / 60.0;
    let longest_focus_minutes = (longest_focus_minutes * 10.0).round() / 10.0;

    // Track distractions: switches FROM productive TO non-productive
    let mut distraction_map: HashMap<String, (u32, f64)> = HashMap::new();
    for i in 1..active_events.len() {
        let prev = active_events[i - 1];
        let curr = active_events[i];
        if prev.productivity_score > 0.0 && curr.productivity_score <= 0.0 {
            let entry = distraction_map.entry(curr.app.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += curr.duration;
        }
    }

    let mut top_distractions: Vec<DistractionEntry> = distraction_map
        .into_iter()
        .map(|(app, (switches_to, total_seconds))| DistractionEntry {
            app,
            switches_to,
            total_seconds,
        })
        .collect();
    top_distractions.sort_by(|a, b| b.switches_to.cmp(&a.switches_to));
    top_distractions.truncate(5);

    // Focus score: deep_work_ratio * 0.7 + (1.0 - switch_penalty) * 0.3
    let deep_work_seconds: f64 = deep_work_blocks.iter().map(|b| b.duration_seconds).sum();
    let deep_work_ratio = if total_active_seconds > 0.0 {
        deep_work_seconds / total_active_seconds
    } else {
        0.0
    };
    let switch_penalty = (switches_per_hour / 30.0).min(1.0);
    let focus_score = ((deep_work_ratio * 0.7 + (1.0 - switch_penalty) * 0.3) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u32;

    Ok(FocusResponse {
        from: from_str,
        to: to_str,
        total_active_seconds,
        total_active_time: format_duration(total_active_seconds),
        focus_score,
        context_switches,
        switches_per_hour,
        deep_work_blocks,
        longest_focus_minutes,
        top_distractions,
    })
}

/// Check if the gap between the end of prev event and start of curr event is <= 65s.
fn check_gap(prev_timestamp: &str, prev_duration: f64, curr_timestamp: &str) -> bool {
    let prev_dt = DateTime::parse_from_rfc3339(prev_timestamp);
    let curr_dt = DateTime::parse_from_rfc3339(curr_timestamp);

    match (prev_dt, curr_dt) {
        (Ok(prev), Ok(curr)) => {
            let prev_end = prev + chrono::Duration::seconds(prev_duration as i64);
            let gap = (curr - prev_end).num_seconds();
            gap <= 65
        }
        _ => true, // If we can't parse timestamps, don't break the block
    }
}
