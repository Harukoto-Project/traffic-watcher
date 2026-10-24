import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { SidebarTrigger } from "@/components/ui/sidebar";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { APP_NAME } from "@/config/app";
import { DEFAULT_PAGE, getPageTitle, type PageId } from "@/config/navigation";

import { ThemeToggle } from "./ThemeToggle";

interface GlobalHeaderProps {
  page: PageId;
  onPageChange: (page: PageId) => void;
}

export function GlobalHeader({ page, onPageChange }: GlobalHeaderProps) {
  return (
    <header className="bg-sidebar flex h-14 items-center gap-4 px-4 md:px-8">
      <Tooltip>
        <TooltipTrigger asChild>
          <SidebarTrigger aria-label="サイドバーを開閉" className="shrink-0" />
        </TooltipTrigger>
        <TooltipContent>サイドバーを開閉</TooltipContent>
      </Tooltip>
      <Breadcrumb className="min-w-0">
        <BreadcrumbList className="flex-nowrap">
          <BreadcrumbItem className="shrink-0 sm:hidden">
            <BreadcrumbEllipsis />
          </BreadcrumbItem>
          <BreadcrumbItem className="hidden shrink-0 sm:inline-flex">
            <BreadcrumbLink asChild>
              <button type="button" onClick={() => onPageChange(DEFAULT_PAGE)}>
                {APP_NAME}
              </button>
            </BreadcrumbLink>
          </BreadcrumbItem>
          <BreadcrumbSeparator />
          <BreadcrumbItem className="min-w-0">
            <BreadcrumbPage className="truncate">
              {getPageTitle(page)}
            </BreadcrumbPage>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>
      <div className="ml-auto shrink-0">
        <ThemeToggle />
      </div>
    </header>
  );
}
