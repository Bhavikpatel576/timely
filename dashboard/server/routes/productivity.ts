import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (req, res) => {
  const from = (req.query.from as string) || new Date().toISOString().slice(0, 10);
  const to = (req.query.to as string) || new Date().toISOString().slice(0, 10);

  const fromDate = `${from}T00:00:00`;
  const toDate = `${to}T23:59:59`;

  const rows = db
    .prepare(
      `SELECT
         COALESCE(c.productivity_score, 0) as score,
         SUM(e.duration) as total_seconds
       FROM events e
       LEFT JOIN categories c ON e.category_id = c.id
       WHERE e.timestamp >= ? AND e.timestamp <= ? AND e.is_afk = 0
       GROUP BY COALESCE(c.productivity_score, 0)`
    )
    .all(fromDate, toDate) as Array<{ score: number; total_seconds: number }>;

  let productive = 0;
  let neutral = 0;
  let distracting = 0;
  let weightedSum = 0;
  let totalSeconds = 0;

  for (const row of rows) {
    totalSeconds += row.total_seconds;
    if (row.score > 0) {
      productive += row.total_seconds;
      weightedSum += row.total_seconds * row.score;
    } else if (row.score < 0) {
      distracting += row.total_seconds;
      weightedSum += row.total_seconds * row.score;
    } else {
      neutral += row.total_seconds;
    }
  }

  // Score: map weighted average from [-2, 2] to [0, 100]
  const weightedAvg = totalSeconds > 0 ? weightedSum / totalSeconds : 0;
  const score = Math.round(((weightedAvg + 2) / 4) * 100);

  res.json({
    score: Math.max(0, Math.min(100, score)),
    productive: Math.round(productive),
    neutral: Math.round(neutral),
    distracting: Math.round(distracting),
    total: Math.round(totalSeconds),
  });
});

export default router;
