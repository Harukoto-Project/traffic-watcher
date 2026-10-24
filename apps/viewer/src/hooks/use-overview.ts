import { loadOverview } from "@/services/overviewService";
import type { Overview } from "@/types/monitor";

import { useAsyncData, type AsyncState } from "./use-async-data";
import { useDatabase } from "./use-database";
import { useTimeRange } from "./use-time-range";

export function useOverview(): AsyncState<Overview> {
  const { range } = useTimeRange();
  const { revision } = useDatabase();
  return useAsyncData(
    () => loadOverview(range.sinceMs, range.untilMs),
    [range.sinceMs, range.untilMs, revision],
  );
}
