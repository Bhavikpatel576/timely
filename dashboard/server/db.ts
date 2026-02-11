import Database from "better-sqlite3";
import path from "path";
import os from "os";

const dbPath = path.join(os.homedir(), ".timely", "timely.db");

const db = new Database(dbPath, { readonly: true });
db.pragma("journal_mode = WAL");
db.pragma("foreign_keys = ON");

export default db;
