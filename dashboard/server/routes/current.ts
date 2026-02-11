import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (_req, res) => {
  const row = db
    .prepare(
      `SELECT
         e.app,
         e.title,
         e.url,
         COALESCE(c.name, 'uncategorized') as category,
         e.duration,
         e.is_afk,
         e.timestamp
       FROM events e
       LEFT JOIN categories c ON e.category_id = c.id
       ORDER BY e.timestamp DESC
       LIMIT 1`
    )
    .get() as
    | {
        app: string | null;
        title: string | null;
        url: string | null;
        category: string;
        duration: number;
        is_afk: number;
        timestamp: string;
      }
    | undefined;

  if (!row) {
    res.json(null);
    return;
  }

  res.json({
    app: row.app,
    title: row.title,
    url: row.url,
    category: row.category,
    duration_seconds: row.duration,
    is_afk: row.is_afk === 1,
    since: row.timestamp,
  });
});

export default router;
