import {
  CalendarClock,
  Camera,
  ChartLine,
  LayoutDashboard,
  ListChecks,
  type LucideIcon,
} from "lucide-react";

export type PageId =
  "overview" | "timeline" | "snapshots" | "events" | "hourly";

export interface NavigationItem {
  id: PageId;
  label: string;
  icon: LucideIcon;
}

export interface NavigationGroup {
  id: string;
  label: string;
  items: NavigationItem[];
}

export const navigationGroups: NavigationGroup[] = [
  {
    id: "status",
    label: "状況",
    items: [
      { id: "overview", label: "概要", icon: LayoutDashboard },
      { id: "timeline", label: "タイムライン", icon: ChartLine },
      { id: "hourly", label: "長期集計", icon: CalendarClock },
    ],
  },
  {
    id: "records",
    label: "記録",
    items: [
      { id: "snapshots", label: "スナップショット", icon: Camera },
      { id: "events", label: "イベント", icon: ListChecks },
    ],
  },
];

export const navigationItems: NavigationItem[] = navigationGroups.flatMap(
  (group) => group.items,
);

export const DEFAULT_PAGE: PageId = "overview";

export function getPageTitle(id: PageId): string {
  return navigationItems.find((item) => item.id === id)?.label ?? "";
}
