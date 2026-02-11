import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (req, res) => {
  const from = (req.query.from as string) || new Date().toISOString().slice(0, 10);
  const to = (req.query.to as string) || new Date().toISOString().slice(0, 10);
  const limit = parseInt((req.query.limit as string) || "20", 10);

  const fromDate = `${from}T00:00:00`;
  const toDate = `${to}T23:59:59`;

  const rows = db
    .prepare(
      `SELECT
         COALESCE(e.app, 'Unknown') as app,
         COALESCE(c.name, 'uncategorized') as category,
         SUM(e.duration) as total_seconds,
         COUNT(*) as event_count
       FROM events e
       LEFT JOIN categories c ON e.category_id = c.id
       WHERE e.timestamp >= ? AND e.timestamp <= ? AND e.is_afk = 0
       GROUP BY e.app
       ORDER BY total_seconds DESC
       LIMIT ?`
    )
    .all(fromDate, toDate, limit) as Array<{
    app: string;
    category: string;
    total_seconds: number;
    event_count: number;
  }>;

  const totalSeconds = rows.reduce((sum, r) => sum + r.total_seconds, 0);

  const apps = rows.map((r) => ({
    app: r.app,
    category: r.category,
    seconds: Math.round(r.total_seconds),
    time: formatDuration(r.total_seconds),
    pct: totalSeconds > 0 ? Math.round((r.total_seconds / totalSeconds) * 1000) / 10 : 0,
    events: r.event_count,
  }));

  res.json(apps);
});

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.round((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export default router;
