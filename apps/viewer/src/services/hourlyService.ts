import type { HourlyRow } from "@/types/monitor";

import { invokeCommand } from "./invokeCommand";

export function listHourly(
  sinceMs: number,
  untilMs: number,
  source: string | null,
): Promise<HourlyRow[]> {
  return invokeCommand<HourlyRow[]>("list_hourly", {
    sinceMs,
    untilMs,
    source,
  });
}
