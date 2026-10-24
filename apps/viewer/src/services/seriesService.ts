import type { SeriesResponse } from "@/types/monitor";

import { invokeCommand } from "./invokeCommand";

export function loadSeries(
  sinceMs: number,
  untilMs: number,
  maxPoints: number,
): Promise<SeriesResponse> {
  return invokeCommand<SeriesResponse>("load_series", {
    sinceMs,
    untilMs,
    maxPoints,
  });
}
