import { loadSeries } from "@/services/seriesService";
import type { SeriesResponse } from "@/types/monitor";

import { useAsyncData, type AsyncState } from "./use-async-data";
import { useDatabase } from "./use-database";
import { useTimeRange } from "./use-time-range";

const MAX_POINTS = 800;

export function useSeries(): AsyncState<SeriesResponse> {
  const { range } = useTimeRange();
  const { revision } = useDatabase();
  return useAsyncData(
    () => loadSeries(range.sinceMs, range.untilMs, MAX_POINTS),
    [range.sinceMs, range.untilMs, revision],
  );
}
