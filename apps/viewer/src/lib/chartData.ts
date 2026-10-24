import type { HourlyMetric, HourlyTransform } from "@/config/hourlyMetrics";
import type {
  HourlyRow,
  MonitorEvent,
  PingBucket,
  SystemBucket,
} from "@/types/monitor";

export interface ChartRow {
  ts: number;
  [key: string]: number | null | undefined;
}

export interface ChartSeries {
  key: string;
  label: string;
  color: string;
  dash?: string;
}

export interface PingTarget {
  key: string;
  source: string;
  label: string;
  color: string;
  dash?: string;
}

export interface PingChartData {
  rows: ChartRow[];
  targets: PingTarget[];
}

export interface HourlyChartData {
  rows: ChartRow[];
  series: ChartSeries[];
}

const PALETTE_SIZE = 5;
const GAP_FACTOR = 3;
const HOUR_MS = 3_600_000;
const MEGABYTE = 1_048_576;

export const MAX_DASH = "1.5 3";

const SERIES_DASHES: (string | undefined)[] = [undefined, "6 3", "10 3 2 3"];

export function chartDash(index: number): string | undefined {
  return SERIES_DASHES[index % SERIES_DASHES.length];
}

export function chartColor(index: number): string {
  return `var(--chart-${(index % PALETTE_SIZE) + 1})`;
}

function round(
  value: number | null | undefined,
  digits: number,
): number | null {
  if (value === null || value === undefined) {
    return null;
  }
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

function toMegabytes(value: number | null | undefined): number | null {
  if (value === null || value === undefined) {
    return null;
  }
  return round(value / MEGABYTE, 3);
}

function insertGaps(rows: ChartRow[], stepMs: number): ChartRow[] {
  const out: ChartRow[] = [];
  rows.forEach((row, index) => {
    const prev = index > 0 ? rows[index - 1] : null;
    if (prev && row.ts - prev.ts > stepMs * GAP_FACTOR) {
      out.push({ ts: prev.ts + stepMs });
    }
    out.push(row);
  });
  return out;
}

function ensureRow(map: Map<number, ChartRow>, ts: number): ChartRow {
  const existing = map.get(ts);
  if (existing) {
    return existing;
  }
  const created: ChartRow = { ts };
  map.set(ts, created);
  return created;
}

function sortedRows(map: Map<number, ChartRow>): ChartRow[] {
  return [...map.values()].sort((a, b) => a.ts - b.ts);
}

function buildTargets(sources: string[]): PingTarget[] {
  return [...new Set(sources)].sort().map((source, index) => ({
    key: `t${index}`,
    source,
    label: source.replace(/^ping:/, ""),
    color: chartColor(index),
    dash: chartDash(index),
  }));
}

export function buildPingChartData(
  buckets: PingBucket[],
  bucketMs: number,
): PingChartData {
  const targets = buildTargets(buckets.map((bucket) => bucket.source));
  const keyBySource = new Map(
    targets.map((target) => [target.source, target.key]),
  );
  const byTs = new Map<number, ChartRow>();

  for (const bucket of buckets) {
    const key = keyBySource.get(bucket.source);
    if (!key) {
      continue;
    }
    const row = ensureRow(byTs, bucket.bucket_ms);
    row[`avg_${key}`] = round(bucket.avg_ms, 2);
    row[`max_${key}`] = round(bucket.max_ms, 2);
    row[`lost_${key}`] = bucket.lost;
  }

  return { rows: insertGaps(sortedRows(byTs), bucketMs), targets };
}

export function buildSystemChartData(
  buckets: SystemBucket[],
  bucketMs: number,
): ChartRow[] {
  const rows = buckets.map((bucket): ChartRow => ({
    ts: bucket.bucket_ms,
    cpu_avg: round(bucket.cpu_avg, 1),
    cpu_max: round(bucket.cpu_max, 1),
    mem_avg: round(bucket.mem_avg, 1),
    mem_max: round(bucket.mem_max, 1),
    rx_avg: toMegabytes(bucket.rx_avg),
    rx_max: toMegabytes(bucket.rx_max),
    tx_avg: toMegabytes(bucket.tx_avg),
    tx_max: toMegabytes(bucket.tx_max),
  }));
  return insertGaps(rows, bucketMs);
}

export function buildSnapshotPingData(events: MonitorEvent[]): PingChartData {
  const pings = events.filter((event) => event.source.startsWith("ping:"));
  const targets = buildTargets(pings.map((event) => event.source));
  const keyBySource = new Map(
    targets.map((target) => [target.source, target.key]),
  );
  const byTs = new Map<number, ChartRow>();

  for (const event of pings) {
    const key = keyBySource.get(event.source);
    if (!key) {
      continue;
    }
    const rtt = event.fields.rtt_ms;
    const row = ensureRow(byTs, event.ts_ms);
    row[`rtt_${key}`] = typeof rtt === "number" ? round(rtt, 2) : null;
  }

  return { rows: sortedRows(byTs), targets };
}

export function buildSnapshotSystemData(events: MonitorEvent[]): ChartRow[] {
  return events
    .filter((event) => event.source === "system")
    .map((event): ChartRow => {
      const cpu = event.fields.cpu_pct;
      const mem = event.fields.mem_used_pct;
      return {
        ts: event.ts_ms,
        cpu: typeof cpu === "number" ? round(cpu, 1) : null,
        mem: typeof mem === "number" ? round(mem, 1) : null,
      };
    })
    .sort((a, b) => a.ts - b.ts);
}

function applyTransform(
  value: number,
  transform: HourlyTransform,
): number | null {
  switch (transform) {
    case "lossPct":
      return round((1 - value) * 100, 2);
    case "megabytes":
      return toMegabytes(value);
    default:
      return round(value, 2);
  }
}

export function buildHourlyChartData(
  hourly: HourlyRow[],
  metric: HourlyMetric,
): HourlyChartData {
  const matching = hourly.filter((row) => {
    if (row.metric !== metric.metric) {
      return false;
    }
    return metric.sourceKind === "ping"
      ? row.source.startsWith("ping:")
      : row.source === "system";
  });

  const sources = [...new Set(matching.map((row) => row.source))].sort();
  const series: ChartSeries[] = [];
  sources.forEach((source, index) => {
    const label = source.replace(/^ping:/, "");
    const color = chartColor(index);
    series.push({
      key: `avg_s${index}`,
      label: metric.sourceKind === "ping" ? label : "平均",
      color,
      dash: chartDash(index),
    });
    if (metric.showMax) {
      series.push({
        key: `max_s${index}`,
        label: metric.sourceKind === "ping" ? `${label}(最大)` : "最大",
        color,
        dash: MAX_DASH,
      });
    }
  });

  const indexBySource = new Map(
    sources.map((source, index) => [source, index]),
  );
  const byTs = new Map<number, ChartRow>();
  for (const row of matching) {
    const index = indexBySource.get(row.source);
    if (index === undefined) {
      continue;
    }
    const target = ensureRow(byTs, row.hour_ms);
    target[`avg_s${index}`] = applyTransform(row.avg, metric.transform);
    if (metric.showMax) {
      target[`max_s${index}`] = applyTransform(row.max, metric.transform);
    }
  }

  return { rows: insertGaps(sortedRows(byTs), HOUR_MS), series };
}
