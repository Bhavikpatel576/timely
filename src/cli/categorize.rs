use crate::db;
use crate::db::categories as db_categories;
use crate::error::{Result, TimelyError};
use crate::output;

pub fn cmd_set(pattern: &str, category: &str, field: &str, retroactive: bool, json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    db_categories::seed_builtin_categories(&conn)?;

    // Find or create category
    let cat = match db_categories::get_category_by_name(&conn, category)? {
        Some(c) => c,
        None => {
            // Auto-create the category
            let parent_id = if category.contains('/') {
                let parent_name = category.split('/').next().unwrap();
                db_categories::get_category_by_name(&conn, parent_name)?
                    .map(|c| c.id)
            } else {
                None
            };
            let id = db_categories::insert_category(&conn, category, parent_id, 0.0)?;
            db_categories::get_category_by_id(&conn, id)?
                .ok_or_else(|| TimelyError::Generic("Failed to create category".into()))?
        }
    };

    // User rules get priority 200 (above builtins)
    db_categories::insert_rule(&conn, cat.id, field, pattern, false, 200)?;

    let mut retroactive_count = 0;
    if retroactive {
        retroactive_count = reclassify_matching(&conn, field, pattern, cat.id)?;
    }

    if json {
        output::print_json(&serde_json::json!({
            "field": field,
            "pattern": pattern,
            "category": category,
            "category_id": cat.id,
            "retroactive_updates": retroactive_count,
        }));
    } else {
        println!("Rule added: {} '{}' -> {}", field, pattern, category);
        if retroactive {
            println!("Retroactively updated {} events", retroactive_count);
        }
    }

    Ok(())
}

pub fn cmd_list(json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    db_categories::seed_builtin_categories(&conn)?;
    let rules = db_categories::list_rules(&conn)?;

    if json {
        output::print_json(&rules);
    } else {
        println!("{:<6} {:<8} {:<25} {:<25} {:<10} {}", "ID", "Builtin", "Pattern", "Category", "Field", "Priority");
        println!("{:-<90}", "");
        for rule in &rules {
            let cat_name = rule.category_name.as_deref().unwrap_or("-");
            println!(
                "{:<6} {:<8} {:<25} {:<25} {:<10} {}",
                rule.id,
                if rule.is_builtin { "yes" } else { "no" },
                rule.pattern,
                cat_name,
                rule.field,
                rule.priority,
            );
        }
    }

    Ok(())
}

pub fn cmd_delete(id: i64, json: bool) -> Result<()> {
    let conn = db::open_default_db()?;
    if db_categories::delete_rule(&conn, id)? {
        if json {
            output::print_json(&serde_json::json!({
                "deleted": true,
                "rule_id": id,
            }));
        } else {
            println!("Rule {} deleted", id);
        }
    } else {
        return Err(TimelyError::RuleNotFound(id));
    }
    Ok(())
}

fn reclassify_matching(conn: &rusqlite::Connection, field: &str, pattern: &str, category_id: i64) -> Result<i64> {
    let sql = match field {
        "app" => "UPDATE events SET category_id = ?1 WHERE LOWER(app) = LOWER(?2)",
        "title" => "UPDATE events SET category_id = ?1 WHERE LOWER(title) = LOWER(?2)",
        "url_domain" => "UPDATE events SET category_id = ?1 WHERE LOWER(url_domain) = LOWER(?2)",
        _ => return Ok(0),
    };

    let count = conn.execute(sql, rusqlite::params![category_id, pattern])?;
    Ok(count as i64)
}
