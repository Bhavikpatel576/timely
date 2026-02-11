import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (req, res) => {
  const from = (req.query.from as string) || new Date().toISOString().slice(0, 10);
  const to = (req.query.to as string) || new Date().toISOString().slice(0, 10);
  const limit = parseInt((req.query.limit as string) || "200", 10);

  const fromDate = `${from}T00:00:00`;
  const toDate = `${to}T23:59:59`;

  const rows = db
    .prepare(
      `SELECT
         e.timestamp,
         e.duration,
         e.app,
         e.title,
         e.url,
         COALESCE(c.name, 'uncategorized') as category,
         e.is_afk
       FROM events e
       LEFT JOIN categories c ON e.category_id = c.id
       WHERE e.timestamp >= ? AND e.timestamp <= ? AND e.is_afk = 0
       ORDER BY e.timestamp ASC
       LIMIT ?`
    )
    .all(fromDate, toDate, limit) as Array<{
    timestamp: string;
    duration: number;
    app: string | null;
    title: string | null;
    url: string | null;
    category: string;
    is_afk: number;
  }>;

  const timeline = rows.map((r) => ({
    timestamp: r.timestamp,
    duration: r.duration,
    app: r.app,
    title: r.title,
    url: r.url,
    category: r.category,
    is_afk: r.is_afk === 1,
  }));

  res.json(timeline);
});

export default router;
