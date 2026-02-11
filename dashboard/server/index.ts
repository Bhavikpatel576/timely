import express from "express";
import cors from "cors";
import path from "path";
import { fileURLToPath } from "url";
import summaryRouter from "./routes/summary.js";
import categoriesRouter from "./routes/categories.js";
import appsRouter from "./routes/apps.js";
import timelineRouter from "./routes/timeline.js";
import productivityRouter from "./routes/productivity.js";
import trendsRouter from "./routes/trends.js";
import currentRouter from "./routes/current.js";
import rulesRouter from "./routes/rules.js";

const app = express();
const PORT = 3123;

app.use(cors());
app.use(express.json());

app.use("/api/summary", summaryRouter);
app.use("/api/categories", categoriesRouter);
app.use("/api/apps", appsRouter);
app.use("/api/timeline", timelineRouter);
app.use("/api/productivity", productivityRouter);
app.use("/api/trends", trendsRouter);
app.use("/api/current", currentRouter);
app.use("/api/rules", rulesRouter);

// In production, serve the built frontend
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distPath = path.join(__dirname, "..", "dist");
app.use(express.static(distPath));
app.get("/{*splat}", (_req, res) => {
  res.sendFile(path.join(distPath, "index.html"));
});

app.listen(PORT, () => {
  console.log(`Timetrack dashboard API running on http://localhost:${PORT}`);
});
