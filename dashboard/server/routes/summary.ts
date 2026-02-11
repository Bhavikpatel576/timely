import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (req, res) => {
  const from = (req.query.from as string) || new Date().toISOString().slice(0, 10);
  const to = (req.query.to as string) || new Date().toISOString().slice(0, 10);
  const groupBy = (req.query.groupBy as string) || "category";

  const fromDate = `${from}T00:00:00`;
  const toDate = `${to}T23:59:59`;

  let rows: Array<{ name: string; total_seconds: number }>;

  if (groupBy === "app") {
    rows = db
      .prepare(
        `SELECT COALESCE(app, 'Unknown') as name, SUM(duration) as total_seconds
         FROM events
         WHERE timestamp >= ? AND timestamp <= ? AND is_afk = 0
         GROUP BY app
         ORDER BY total_seconds DESC`
      )
      .all(fromDate, toDate) as typeof rows;
  } else {
    rows = db
      .prepare(
        `SELECT COALESCE(c.name, 'uncategorized') as name, SUM(e.duration) as total_seconds
         FROM events e
         LEFT JOIN categories c ON e.category_id = c.id
         WHERE e.timestamp >= ? AND e.timestamp <= ? AND e.is_afk = 0
         GROUP BY COALESCE(c.name, 'uncategorized')
         ORDER BY total_seconds DESC`
      )
      .all(fromDate, toDate) as typeof rows;
  }

  const totalSeconds = rows.reduce((sum, r) => sum + r.total_seconds, 0);

  const groups = rows.map((r) => ({
    name: r.name,
    seconds: Math.round(r.total_seconds),
    time: formatDuration(r.total_seconds),
    pct: totalSeconds > 0 ? Math.round((r.total_seconds / totalSeconds) * 1000) / 10 : 0,
  }));

  res.json({
    period_from: fromDate,
    period_to: toDate,
    total_active: formatDuration(totalSeconds),
    total_active_seconds: Math.round(totalSeconds),
    groups,
  });
});

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.round((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export default router;
