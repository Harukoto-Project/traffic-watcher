import { useCallback, useMemo, useState, type ReactNode } from "react";

import { useDatabase } from "@/hooks/use-database";
import { defaultSelection, resolveRange } from "@/lib/timeRange";
import type {
  DataBounds,
  RangePresetId,
  RangeSelection,
} from "@/types/timeRange";

import { TimeRangeContext } from "./timeRangeContext";

export function TimeRangeProvider({ children }: { children: ReactNode }) {
  const { info } = useDatabase();
  const [selection, setSelection] = useState<RangeSelection>(defaultSelection);

  const firstMs = info?.first_ms ?? null;
  const lastMs = info?.last_ms ?? null;
  const bounds = useMemo<DataBounds | null>(
    () => (firstMs !== null && lastMs !== null ? { firstMs, lastMs } : null),
    [firstMs, lastMs],
  );

  const range = useMemo(
    () => resolveRange(selection, bounds),
    [selection, bounds],
  );

  const setPreset = useCallback(
    (preset: RangePresetId) => {
      setSelection((prev) => {
        if (preset !== "custom") {
          return { ...prev, preset };
        }
        const current = resolveRange(prev, bounds);
        return {
          preset,
          customSinceMs: prev.customSinceMs ?? current.sinceMs,
          customUntilMs: prev.customUntilMs ?? current.untilMs,
        };
      });
    },
    [bounds],
  );

  const setCustom = useCallback(
    (customSinceMs: number, customUntilMs: number) => {
      setSelection({ preset: "custom", customSinceMs, customUntilMs });
    },
    [],
  );

  const value = useMemo(
    () => ({ selection, range, bounds, setPreset, setCustom }),
    [selection, range, bounds, setPreset, setCustom],
  );

  return (
    <TimeRangeContext.Provider value={value}>
      {children}
    </TimeRangeContext.Provider>
  );
}
