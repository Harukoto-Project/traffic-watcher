import type { Overview } from "@/types/monitor";

import { invokeCommand } from "./invokeCommand";

export function loadOverview(
  sinceMs: number,
  untilMs: number,
): Promise<Overview> {
  return invokeCommand<Overview>("load_overview", { sinceMs, untilMs });
}
