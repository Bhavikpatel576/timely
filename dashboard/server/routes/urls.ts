import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (req, res) => {
  const from = (req.query.from as string) || new Date().toISOString().slice(0, 10);
  const to = (req.query.to as string) || new Date().toISOString().slice(0, 10);
  const search = (req.query.search as string) || "";
  const domain = (req.query.domain as string) || "";
  const category = (req.query.category as string) || "";
  const page = Math.max(1, parseInt((req.query.page as string) || "1", 10));
  const limit = Math.min(100, Math.max(1, parseInt((req.query.limit as string) || "50", 10)));
  const sort = (req.query.sort as string) || "timestamp";
  const order = (req.query.order as string) === "asc" ? "ASC" : "DESC";

  const fromDate = `${from}T00:00:00`;
  const toDate = `${to}T23:59:59`;

  // Validate sort column to prevent SQL injection
  const allowedSorts: Record<string, string> = {
    timestamp: "e.timestamp",
    duration: "e.duration",
    app: "e.app",
    url_domain: "e.url_domain",
    category: "category",
  };
  const sortCol = allowedSorts[sort] || "e.timestamp";

  // Build WHERE clauses
  const conditions = [
    "e.timestamp >= ?",
    "e.timestamp <= ?",
    "e.url IS NOT NULL",
    "e.url != ''",
  ];
  const params: (string | number)[] = [fromDate, toDate];

  if (search) {
    conditions.push("(e.url LIKE ? OR e.url_domain LIKE ? OR e.title LIKE ?)");
    const like = `%${search}%`;
    params.push(like, like, like);
  }
  if (domain) {
    conditions.push("e.url_domain = ?");
    params.push(domain);
  }
  if (category) {
    conditions.push("e.category_id = ?");
    params.push(parseInt(category, 10));
  }

  const where = conditions.join(" AND ");

  // Count total matching rows
  const countRow = db
    .prepare(`SELECT COUNT(*) as total FROM events e WHERE ${where}`)
    .get(...params) as { total: number };

  // Fetch paginated rows
  const offset = (page - 1) * limit;
  const rows = db
    .prepare(
      `SELECT
         e.url,
         e.url_domain,
         e.title,
         e.timestamp,
         e.duration,
         COALESCE(c.name, 'uncategorized') as category,
         e.app,
         e.is_afk
       FROM events e
       LEFT JOIN categories c ON e.category_id = c.id
       WHERE ${where}
       ORDER BY ${sortCol} ${order}
       LIMIT ? OFFSET ?`
    )
    .all(...params, limit, offset) as Array<{
    url: string;
    url_domain: string | null;
    title: string | null;
    timestamp: string;
    duration: number;
    category: string;
    app: string | null;
    is_afk: number;
  }>;

  // Get distinct domains for filter dropdown (within date range)
  const domains = db
    .prepare(
      `SELECT DISTINCT url_domain FROM events
       WHERE timestamp >= ? AND timestamp <= ?
         AND url IS NOT NULL AND url != '' AND url_domain IS NOT NULL AND url_domain != ''
       ORDER BY url_domain ASC`
    )
    .all(fromDate, toDate) as Array<{ url_domain: string }>;

  res.json({
    rows: rows.map((r) => ({
      url: r.url,
      url_domain: r.url_domain,
      title: r.title,
      timestamp: r.timestamp,
      duration: r.duration,
      category: r.category,
      app: r.app,
      is_afk: r.is_afk === 1,
    })),
    total: countRow.total,
    page,
    limit,
    domains: domains.map((d) => d.url_domain),
  });
});

export default router;
