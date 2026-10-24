import { useState, type ReactNode } from "react";

import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { PageId } from "@/config/navigation";
import { PageHeaderSlotContext } from "@/contexts/pageHeaderSlotContext";

import { AppSidebar } from "./AppSidebar";
import { GlobalHeader } from "./GlobalHeader";

interface AppShellProps {
  page: PageId;
  onPageChange: (page: PageId) => void;
  children: ReactNode;
}

export function AppShell({ page, onPageChange, children }: AppShellProps) {
  const [pageHeaderSlot, setPageHeaderSlot] = useState<HTMLDivElement | null>(
    null,
  );

  return (
    <TooltipProvider>
      <SidebarProvider>
        <AppSidebar page={page} onPageChange={onPageChange} />
        <SidebarInset className="min-w-0">
          <div className="sticky top-0 z-10 flex flex-col">
            <GlobalHeader page={page} onPageChange={onPageChange} />
            <div
              ref={setPageHeaderSlot}
              className="bg-card border-b px-4 py-3 empty:hidden md:px-8"
            />
          </div>
          <PageHeaderSlotContext.Provider value={pageHeaderSlot}>
            <div className="flex flex-1 flex-col px-4 py-6 md:px-8 md:py-8">
              <div className="mx-auto flex w-full max-w-6xl flex-col gap-8">
                {children}
              </div>
            </div>
          </PageHeaderSlotContext.Provider>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
