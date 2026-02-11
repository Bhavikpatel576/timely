use rusqlite::Connection;
use crate::error::Result;
use crate::types::{Category, CategoryRule};

pub fn insert_category(conn: &Connection, name: &str, parent_id: Option<i64>, score: f64) -> Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO categories (name, parent_id, productivity_score) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, parent_id, score],
    )?;
    let id = conn.query_row(
        "SELECT id FROM categories WHERE name = ?1",
        rusqlite::params![name],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn get_category_by_name(conn: &Connection, name: &str) -> Result<Option<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, productivity_score FROM categories WHERE name = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            productivity_score: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_category_by_id(conn: &Connection, id: i64) -> Result<Option<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, productivity_score FROM categories WHERE id = ?1",
    )?;
    let mut rows = stmt.query(rusqlite::params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            productivity_score: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, parent_id, productivity_score FROM categories ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
            productivity_score: row.get(3)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn insert_rule(
    conn: &Connection,
    category_id: i64,
    field: &str,
    pattern: &str,
    is_builtin: bool,
    priority: i32,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO category_rules (category_id, field, pattern, is_builtin, priority)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![category_id, field, pattern, is_builtin as i32, priority],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_rules(conn: &Connection) -> Result<Vec<CategoryRule>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.category_id, c.name, r.field, r.pattern, r.is_builtin, r.priority
         FROM category_rules r
         JOIN categories c ON c.id = r.category_id
         ORDER BY r.priority DESC, r.id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CategoryRule {
            id: row.get(0)?,
            category_id: row.get(1)?,
            category_name: row.get(2)?,
            field: row.get(3)?,
            pattern: row.get(4)?,
            is_builtin: row.get::<_, i32>(5)? != 0,
            priority: row.get(6)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn delete_rule(conn: &Connection, rule_id: i64) -> Result<bool> {
    let changed = conn.execute(
        "DELETE FROM category_rules WHERE id = ?1",
        rusqlite::params![rule_id],
    )?;
    Ok(changed > 0)
}

pub fn seed_builtin_categories(conn: &Connection) -> Result<()> {
    use crate::categories::builtin::BUILTIN_CATEGORIES;

    for (name, parent_name, score) in BUILTIN_CATEGORIES {
        let parent_id = if let Some(pname) = parent_name {
            get_category_by_name(conn, pname)?.map(|c| c.id)
        } else {
            None
        };
        insert_category(conn, name, parent_id, *score)?;
    }

    use crate::categories::builtin::BUILTIN_RULES;

    // Collect the canonical set of (field, pattern) from the source code
    let canonical: std::collections::HashSet<(&str, &str)> = BUILTIN_RULES
        .iter()
        .map(|(_, field, pattern, _)| (*field, *pattern))
        .collect();

    // Remove stale builtin rules that are no longer in the source
    let stale_ids: Vec<i64> = {
        let mut stmt = conn.prepare(
            "SELECT id, field, pattern FROM category_rules WHERE is_builtin = 1",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut ids = Vec::new();
        for row in rows {
            let (id, field, pattern) = row?;
            if !canonical.contains(&(field.as_str(), pattern.as_str())) {
                ids.push(id);
            }
        }
        ids
    };
    for id in stale_ids {
        conn.execute(
            "DELETE FROM category_rules WHERE id = ?1",
            rusqlite::params![id],
        )?;
    }

    // Insert any missing builtin rules (idempotent)
    for (category_name, field, pattern, priority) in BUILTIN_RULES {
        if let Some(cat) = get_category_by_name(conn, category_name)? {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM category_rules WHERE field = ?1 AND pattern = ?2 AND is_builtin = 1",
                    rusqlite::params![field, pattern],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            if !exists {
                insert_rule(conn, cat.id, field, pattern, true, *priority)?;
            }
        }
    }
    Ok(())
}
