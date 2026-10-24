import { open } from "@tauri-apps/plugin-dialog";

import type { DatabaseInfo } from "@/types/database";

import { toAppError } from "./AppError";
import { invokeCommand } from "./invokeCommand";

export function openDatabase(path: string): Promise<DatabaseInfo> {
  return invokeCommand<DatabaseInfo>("open_database", { path });
}

export function closeDatabase(): Promise<void> {
  return invokeCommand<void>("close_database");
}

export function loadDatabaseInfo(): Promise<DatabaseInfo | null> {
  return invokeCommand<DatabaseInfo | null>("get_database_info");
}

export function findDefaultDatabase(): Promise<string | null> {
  return invokeCommand<string | null>("find_default_database");
}

export async function pickDatabaseFile(): Promise<string | null> {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "SQLite", extensions: ["db", "sqlite", "sqlite3"] }],
    });
    return typeof selected === "string" ? selected : null;
  } catch (err) {
    throw toAppError(err);
  }
}
