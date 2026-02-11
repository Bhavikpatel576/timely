import Database from "better-sqlite3";
import path from "path";
import os from "os";

const dbPath = path.join(os.homedir(), ".timely", "timely.db");

const dbWriter = new Database(dbPath);
dbWriter.pragma("journal_mode = WAL");
dbWriter.pragma("foreign_keys = ON");

export default dbWriter;
