import { listHourly } from "@/services/hourlyService";
import type { HourlyRow } from "@/types/monitor";

import { useAsyncData, type AsyncState } from "./use-async-data";
import { useDatabase } from "./use-database";
import { useTimeRange } from "./use-time-range";

export function useHourly(): AsyncState<HourlyRow[]> {
  const { range } = useTimeRange();
  const { revision } = useDatabase();
  return useAsyncData(
    () => listHourly(range.sinceMs, range.untilMs, null),
    [range.sinceMs, range.untilMs, revision],
  );
}
