export type FieldValue = boolean | number | string;

export type EventKind = "metric" | "alert" | "info";

export interface MonitorEvent {
  ts_ms: number;
  source: string;
  kind: EventKind;
  fields: Record<string, FieldValue>;
}

export interface SourceSummary {
  source: string;
  samples: number;
  lost: number;
  loss_pct: number;
  avg_ms: number | null;
  p95_ms: number | null;
  p99_ms: number | null;
  max_ms: number | null;
}

export interface SystemSummary {
  samples: number;
  cpu_avg: number | null;
  cpu_max: number | null;
  mem_avg: number | null;
  mem_max: number | null;
  rx_avg: number | null;
  rx_max: number | null;
  tx_avg: number | null;
  tx_max: number | null;
}

export interface Overview {
  ping: SourceSummary[];
  system: SystemSummary;
  alert_count: number;
}

export interface PingBucket {
  bucket_ms: number;
  source: string;
  samples: number;
  lost: number;
  avg_ms: number | null;
  max_ms: number | null;
}

export interface SystemBucket {
  bucket_ms: number;
  cpu_avg: number | null;
  cpu_max: number | null;
  mem_avg: number | null;
  mem_max: number | null;
  rx_avg: number | null;
  rx_max: number | null;
  tx_avg: number | null;
  tx_max: number | null;
}

export interface SeriesResponse {
  bucket_ms: number;
  ping: PingBucket[];
  system: SystemBucket[];
}

export interface SnapshotInfo {
  id: number;
  created_ms: number;
  rule: string;
  source: string;
  trigger_ms: number;
  window_start_ms: number;
  window_end_ms: number;
  message: string;
  event_count: number;
}

export interface SnapshotDetail {
  info: SnapshotInfo;
  events: MonitorEvent[];
}

export interface HourlyRow {
  source: string;
  hour_ms: number;
  metric: string;
  samples: number;
  avg: number;
  min: number;
  max: number;
}
