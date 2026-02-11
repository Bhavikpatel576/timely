import { Router } from "express";
import db from "../db.js";

const router = Router();

router.get("/", (_req, res) => {
  const categories = db
    .prepare(
      `SELECT id, name, parent_id, productivity_score FROM categories ORDER BY name`
    )
    .all();

  res.json(categories);
});

export default router;
