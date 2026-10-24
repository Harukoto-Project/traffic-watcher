import { Activity } from "lucide-react";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from "@/components/ui/sidebar";
import { APP_NAME } from "@/config/app";
import type { PageId } from "@/config/navigation";

import { DatabaseBar } from "./DatabaseBar";
import { Navigation } from "./Navigation";

interface AppSidebarProps {
  page: PageId;
  onPageChange: (page: PageId) => void;
}

export function AppSidebar({ page, onPageChange }: AppSidebarProps) {
  return (
    <Sidebar collapsible="icon" variant="sidebar">
      <SidebarHeader className="h-14 justify-center">
        <div className="flex h-8 items-center gap-2 px-2">
          <Activity aria-hidden="true" className="size-4 shrink-0" />
          <span className="text-sidebar-accent-foreground truncate text-sm font-semibold group-data-[collapsible=icon]:hidden">
            {APP_NAME}
          </span>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <Navigation page={page} onChange={onPageChange} />
      </SidebarContent>
      <SidebarFooter className="border-border border-t">
        <DatabaseBar />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
