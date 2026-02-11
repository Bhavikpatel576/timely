import { Router } from "express";
import dbWriter from "../db-writer.js";

const router = Router();

interface RuleRow {
  id: number;
  category_id: number;
  category_name: string;
  field: string;
  pattern: string;
  is_builtin: number;
  priority: number;
}

router.get("/", (_req, res) => {
  const rules = dbWriter
    .prepare(
      `SELECT cr.*, c.name as category_name
       FROM category_rules cr
       LEFT JOIN categories c ON cr.category_id = c.id
       ORDER BY cr.priority DESC, cr.id`
    )
    .all() as RuleRow[];

  res.json(
    rules.map((r) => ({
      ...r,
      is_builtin: Boolean(r.is_builtin),
    }))
  );
});

router.put("/:id", (req, res) => {
  const ruleId = Number(req.params.id);
  const { category_id } = req.body as { category_id: number };

  if (category_id == null) {
    res.status(400).json({ error: "Missing required field: category_id" });
    return;
  }

  const rule = dbWriter
    .prepare(`SELECT * FROM category_rules WHERE id = ?`)
    .get(ruleId) as RuleRow | undefined;

  if (!rule) {
    res.status(404).json({ error: "Rule not found" });
    return;
  }
  if (rule.is_builtin) {
    res.status(400).json({ error: "Cannot modify builtin rules" });
    return;
  }

  dbWriter
    .prepare(`UPDATE category_rules SET category_id = ? WHERE id = ?`)
    .run(category_id, ruleId);

  // Recategorize matching events
  let updateResult;
  if (rule.field === "app") {
    updateResult = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE app = ? AND is_afk = 0`)
      .run(category_id, rule.pattern);
  } else if (rule.field === "url_domain") {
    updateResult = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE url_domain = ? AND is_afk = 0`)
      .run(category_id, rule.pattern);
  } else {
    updateResult = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE title LIKE ? AND is_afk = 0`)
      .run(category_id, `%${rule.pattern}%`);
  }

  res.json({ success: true, updated: updateResult.changes });
});

router.delete("/:id", (req, res) => {
  const ruleId = Number(req.params.id);

  const rule = dbWriter
    .prepare(`SELECT * FROM category_rules WHERE id = ?`)
    .get(ruleId) as RuleRow | undefined;

  if (!rule) {
    res.status(404).json({ error: "Rule not found" });
    return;
  }
  if (rule.is_builtin) {
    res.status(400).json({ error: "Cannot delete builtin rules" });
    return;
  }

  // Find the matching builtin rule for the same field+pattern
  const builtinMatch = dbWriter
    .prepare(
      `SELECT category_id FROM category_rules
       WHERE field = ? AND pattern = ? AND is_builtin = 1
       LIMIT 1`
    )
    .get(rule.field, rule.pattern) as { category_id: number } | undefined;

  // Fallback to "uncategorized" category
  let fallbackCategoryId: number | null = null;
  if (builtinMatch) {
    fallbackCategoryId = builtinMatch.category_id;
  } else {
    const uncategorized = dbWriter
      .prepare(`SELECT id FROM categories WHERE name = 'uncategorized' LIMIT 1`)
      .get() as { id: number } | undefined;
    fallbackCategoryId = uncategorized?.id ?? null;
  }

  // Recategorize affected events
  let recategorized;
  if (rule.field === "app") {
    recategorized = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE app = ? AND is_afk = 0`)
      .run(fallbackCategoryId, rule.pattern);
  } else if (rule.field === "url_domain") {
    recategorized = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE url_domain = ? AND is_afk = 0`)
      .run(fallbackCategoryId, rule.pattern);
  } else {
    recategorized = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE title LIKE ? AND is_afk = 0`)
      .run(fallbackCategoryId, `%${rule.pattern}%`);
  }

  // Delete the rule
  dbWriter.prepare(`DELETE FROM category_rules WHERE id = ?`).run(ruleId);

  res.json({ success: true, recategorized: recategorized.changes });
});

router.post("/", (req, res) => {
  const { app, category_id, field } = req.body as {
    app: string;
    category_id: number;
    field: "app" | "title" | "url_domain";
  };

  if (!app || category_id == null || !field) {
    res.status(400).json({ error: "Missing required fields: app, category_id, field" });
    return;
  }

  if (!["app", "title", "url_domain"].includes(field)) {
    res.status(400).json({ error: "field must be one of: app, title, url_domain" });
    return;
  }

  const pattern = app;

  const existing = dbWriter
    .prepare(
      `SELECT id FROM category_rules WHERE field = ? AND pattern = ? AND is_builtin = 0`
    )
    .get(field, pattern) as { id: number } | undefined;

  if (existing) {
    dbWriter
      .prepare(`UPDATE category_rules SET category_id = ? WHERE id = ?`)
      .run(category_id, existing.id);
  } else {
    dbWriter
      .prepare(
        `INSERT INTO category_rules (category_id, field, pattern, is_builtin, priority)
         VALUES (?, ?, ?, 0, 100)`
      )
      .run(category_id, field, pattern);
  }

  // Recategorize existing events based on field type
  let updateResult;
  if (field === "app") {
    updateResult = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE app = ? AND is_afk = 0`)
      .run(category_id, pattern);
  } else if (field === "url_domain") {
    updateResult = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE url_domain = ? AND is_afk = 0`)
      .run(category_id, pattern);
  } else {
    // title — use LIKE for partial matching
    updateResult = dbWriter
      .prepare(`UPDATE events SET category_id = ? WHERE title LIKE ? AND is_afk = 0`)
      .run(category_id, `%${pattern}%`);
  }

  res.json({ success: true, updated: updateResult.changes });
});

export default router;
