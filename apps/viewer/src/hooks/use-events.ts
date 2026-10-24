import { listEvents } from "@/services/eventService";
import type { EventKind, MonitorEvent } from "@/types/monitor";

import { useAsyncData, type AsyncState } from "./use-async-data";
import { useDatabase } from "./use-database";
import { useTimeRange } from "./use-time-range";

export interface EventFilter {
  source: string | null;
  kind: EventKind | null;
  limit: number;
}

export function useEvents(filter: EventFilter): AsyncState<MonitorEvent[]> {
  const { range } = useTimeRange();
  const { revision } = useDatabase();
  return useAsyncData(
    () =>
      listEvents({
        sinceMs: range.sinceMs,
        untilMs: range.untilMs,
        source: filter.source,
        kind: filter.kind,
        limit: filter.limit,
      }),
    [
      range.sinceMs,
      range.untilMs,
      filter.source,
      filter.kind,
      filter.limit,
      revision,
    ],
  );
}
