export interface TimeRange {
  sinceMs: number;
  untilMs: number;
}

export type RangePresetId =
  "15m" | "1h" | "6h" | "24h" | "7d" | "all" | "custom";

export interface RangeSelection {
  preset: RangePresetId;
  customSinceMs: number | null;
  customUntilMs: number | null;
}

export interface DataBounds {
  firstMs: number;
  lastMs: number;
}
