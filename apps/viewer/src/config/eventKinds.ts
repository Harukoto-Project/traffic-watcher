import type { StatusLevel } from "@/config/status";
import type { EventKind } from "@/types/monitor";

export const EVENT_KIND_LABELS: Record<EventKind, string> = {
  metric: "メトリクス",
  alert: "アラート",
  info: "情報",
};

export const EVENT_KIND_LEVELS: Record<EventKind, StatusLevel> = {
  metric: "normal",
  alert: "critical",
  info: "normal",
};

export const eventKindOptions: { value: EventKind; label: string }[] = (
  Object.keys(EVENT_KIND_LABELS) as EventKind[]
).map((value) => ({ value, label: EVENT_KIND_LABELS[value] }));
