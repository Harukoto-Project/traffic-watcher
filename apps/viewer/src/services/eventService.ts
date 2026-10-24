import type { EventKind, MonitorEvent } from "@/types/monitor";

import { invokeCommand } from "./invokeCommand";

export interface EventQuery {
  sinceMs: number;
  untilMs: number;
  source: string | null;
  kind: EventKind | null;
  limit: number;
}

export function listEvents(query: EventQuery): Promise<MonitorEvent[]> {
  return invokeCommand<MonitorEvent[]>("list_events", { ...query });
}
