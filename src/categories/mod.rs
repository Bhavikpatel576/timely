pub mod builtin;

use crate::types::{CategoryRule, WatcherSnapshot};

pub fn classify(snapshot: &WatcherSnapshot, rules: &[CategoryRule]) -> Option<i64> {
    // Rules are expected to be sorted by priority DESC already
    for rule in rules {
        let value = match rule.field.as_str() {
            "app" => &snapshot.app,
            "title" => &snapshot.title,
            "url_domain" => {
                if let Some(ref d) = snapshot.url_domain {
                    d
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        if matches_pattern(value, &rule.pattern) {
            return Some(rule.category_id);
        }
    }
    None
}

fn matches_pattern(value: &str, pattern: &str) -> bool {
    // Case-insensitive exact match or glob match
    let value_lower = value.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if pattern.contains('*') || pattern.contains('?') {
        glob_match(&value_lower, &pattern_lower)
    } else {
        // Exact match (case-insensitive)
        value_lower == pattern_lower
    }
}

fn glob_match(value: &str, pattern: &str) -> bool {
    let glob_pattern = glob::Pattern::new(pattern);
    match glob_pattern {
        Ok(p) => p.matches(value),
        Err(_) => false,
    }
}
