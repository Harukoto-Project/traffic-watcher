import { OctagonAlert, TriangleAlert, type LucideIcon } from "lucide-react";

export type StatusLevel = "critical" | "warning" | "normal";

interface StatusStyle {
  textClass: string;
  color: string;
  icon: LucideIcon | null;
}

export const STATUS_STYLES: Record<StatusLevel, StatusStyle> = {
  critical: {
    textClass: "text-destructive font-semibold",
    color: "var(--destructive)",
    icon: OctagonAlert,
  },
  warning: {
    textClass: "text-warning font-semibold",
    color: "var(--warning)",
    icon: TriangleAlert,
  },
  normal: {
    textClass: "",
    color: "var(--muted-foreground)",
    icon: null,
  },
};

export const LOSS_WARNING_PCT = 1;

export function lossLevel(lossPct: number): StatusLevel {
  return lossPct >= LOSS_WARNING_PCT ? "warning" : "normal";
}
