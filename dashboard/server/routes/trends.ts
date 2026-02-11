import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (req, res) => {
  const from = (req.query.from as string) || new Date().toISOString().slice(0, 10);
  const to = (req.query.to as string) || new Date().toISOString().slice(0, 10);
  const interval = (req.query.interval as string) || "day";

  const fromDate = `${from}T00:00:00`;
  const toDate = `${to}T23:59:59`;

  let dateExpr: string;
  switch (interval) {
    case "week":
      dateExpr = "strftime('%Y-W%W', e.timestamp)";
      break;
    case "month":
      dateExpr = "strftime('%Y-%m', e.timestamp)";
      break;
    default:
      dateExpr = "date(e.timestamp)";
  }

  const rows = db
    .prepare(
      `SELECT
         ${dateExpr} as bucket,
         COALESCE(c.name, 'uncategorized') as category,
         COALESCE(c.productivity_score, 0) as prod_score,
         SUM(e.duration) as total_seconds
       FROM events e
       LEFT JOIN categories c ON e.category_id = c.id
       WHERE e.timestamp >= ? AND e.timestamp <= ? AND e.is_afk = 0
       GROUP BY bucket, category
       ORDER BY bucket ASC`
    )
    .all(fromDate, toDate) as Array<{
    bucket: string;
    category: string;
    prod_score: number;
    total_seconds: number;
  }>;

  // Pivot into per-bucket objects
  const bucketMap = new Map<
    string,
    { bucket: string; total: number; categories: Record<string, number>; weightedSum: number }
  >();

  for (const row of rows) {
    let entry = bucketMap.get(row.bucket);
    if (!entry) {
      entry = { bucket: row.bucket, total: 0, categories: {}, weightedSum: 0 };
      bucketMap.set(row.bucket, entry);
    }
    entry.total += row.total_seconds;
    entry.categories[row.category] = (entry.categories[row.category] || 0) + row.total_seconds;
    entry.weightedSum += row.total_seconds * row.prod_score;
  }

  const trends = Array.from(bucketMap.values()).map((entry) => ({
    bucket: entry.bucket,
    total_seconds: Math.round(entry.total),
    total_hours: Math.round((entry.total / 3600) * 10) / 10,
    productivity:
      entry.total > 0
        ? Math.max(0, Math.min(100, Math.round(((entry.weightedSum / entry.total + 2) / 4) * 100)))
        : 50,
    categories: entry.categories,
  }));

  res.json(trends);
});

export default router;
