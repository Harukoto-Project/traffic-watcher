import { DEFAULT_PRESET, rangePresets } from "@/config/timeRangePresets";
import type { DataBounds, RangeSelection, TimeRange } from "@/types/timeRange";

const FALLBACK_SPAN_MS = 3_600_000;

export function defaultSelection(): RangeSelection {
  return { preset: DEFAULT_PRESET, customSinceMs: null, customUntilMs: null };
}

export function resolveRange(
  selection: RangeSelection,
  bounds: DataBounds | null,
): TimeRange {
  const lastMs = bounds?.lastMs ?? Date.now();
  const firstMs = bounds?.firstMs ?? lastMs - FALLBACK_SPAN_MS;

  if (selection.preset === "all") {
    return { sinceMs: firstMs, untilMs: lastMs };
  }

  if (selection.preset === "custom") {
    const { customSinceMs, customUntilMs } = selection;
    if (
      customSinceMs !== null &&
      customUntilMs !== null &&
      customSinceMs < customUntilMs
    ) {
      return { sinceMs: customSinceMs, untilMs: customUntilMs };
    }
    return { sinceMs: lastMs - FALLBACK_SPAN_MS, untilMs: lastMs };
  }

  const preset = rangePresets.find((item) => item.id === selection.preset);
  const duration = preset?.durationMs ?? FALLBACK_SPAN_MS;
  return { sinceMs: lastMs - duration, untilMs: lastMs };
}
