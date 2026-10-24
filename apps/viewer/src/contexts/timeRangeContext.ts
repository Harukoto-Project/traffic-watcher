import { createContext } from "react";

import type {
  DataBounds,
  RangePresetId,
  RangeSelection,
  TimeRange,
} from "@/types/timeRange";

export interface TimeRangeContextValue {
  selection: RangeSelection;
  range: TimeRange;
  bounds: DataBounds | null;
  setPreset: (preset: RangePresetId) => void;
  setCustom: (sinceMs: number, untilMs: number) => void;
}

export const TimeRangeContext = createContext<TimeRangeContextValue | null>(
  null,
);
