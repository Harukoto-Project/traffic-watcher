export interface DatabaseInfo {
  path: string;
  size_bytes: number;
  events: number;
  snapshots: number;
  hourly_rows: number;
  first_ms: number | null;
  last_ms: number | null;
  sources: string[];
}
