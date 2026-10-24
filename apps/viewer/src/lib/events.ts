import type { FieldValue, MonitorEvent } from "@/types/monitor";

export interface ProcessEntry {
  rank: number;
  name: string;
  pid: number | null;
  value: number | null;
}

export type ProcessRanking = "cpu" | "mem";

function numberField(
  fields: Record<string, FieldValue>,
  key: string,
): number | null {
  const value = fields[key];
  return typeof value === "number" ? value : null;
}

export function extractTopProcesses(
  event: MonitorEvent,
  ranking: ProcessRanking,
): ProcessEntry[] {
  const valueKey = ranking === "cpu" ? "pct" : "mb";
  const entries: ProcessEntry[] = [];
  for (let rank = 1; ; rank += 1) {
    const name = event.fields[`${ranking}_top${rank}_name`];
    if (typeof name !== "string") {
      break;
    }
    entries.push({
      rank,
      name,
      pid: numberField(event.fields, `${ranking}_top${rank}_pid`),
      value: numberField(event.fields, `${ranking}_top${rank}_${valueKey}`),
    });
  }
  return entries;
}

export function findNearestEvent(
  events: MonitorEvent[],
  source: string,
  tsMs: number,
): MonitorEvent | null {
  let best: MonitorEvent | null = null;
  for (const event of events) {
    if (event.source !== source) {
      continue;
    }
    if (
      best === null ||
      Math.abs(event.ts_ms - tsMs) < Math.abs(best.ts_ms - tsMs)
    ) {
      best = event;
    }
  }
  return best;
}

export function formatFieldValue(value: FieldValue): string {
  if (typeof value === "number" && !Number.isInteger(value)) {
    return value.toFixed(2);
  }
  return String(value);
}

export function summarizeFields(fields: Record<string, FieldValue>): string {
  return Object.entries(fields)
    .map(([key, value]) => `${key}=${formatFieldValue(value)}`)
    .join("  ");
}

export function sourceGroups(sources: string[]): string[] {
  const prefixes = new Set<string>();
  for (const source of sources) {
    const index = source.indexOf(":");
    if (index > 0) {
      prefixes.add(`${source.slice(0, index)}:*`);
    }
  }
  return [...prefixes].sort();
}
