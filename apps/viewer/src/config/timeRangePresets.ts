import type { RangePresetId } from "@/types/timeRange";

export interface RangePreset {
  id: RangePresetId;
  label: string;
  durationMs: number | null;
}

const MINUTE_MS = 60_000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;

export const rangePresets: RangePreset[] = [
  { id: "15m", label: "15分", durationMs: 15 * MINUTE_MS },
  { id: "1h", label: "1時間", durationMs: HOUR_MS },
  { id: "6h", label: "6時間", durationMs: 6 * HOUR_MS },
  { id: "24h", label: "24時間", durationMs: DAY_MS },
  { id: "7d", label: "7日", durationMs: 7 * DAY_MS },
  { id: "all", label: "全期間", durationMs: null },
  { id: "custom", label: "指定", durationMs: null },
];

export const DEFAULT_PRESET: RangePresetId = "1h";
