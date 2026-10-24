import { FolderOpen, RefreshCw } from "lucide-react";

import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { useDatabase } from "@/hooks/use-database";
import { fileName } from "@/lib/format";

export function DatabaseBar() {
  const { info, openWithDialog, reload } = useDatabase();

  return (
    <SidebarMenu>
      {info ? (
        <SidebarMenuItem className="group-data-[collapsible=icon]:hidden">
          <p
            className="text-muted-foreground truncate px-2 py-1 text-xs"
            title={info.path}
          >
            {fileName(info.path)}
          </p>
        </SidebarMenuItem>
      ) : null}
      <SidebarMenuItem>
        <SidebarMenuButton
          tooltip="データベースを開く"
          onClick={() => void openWithDialog()}
        >
          <FolderOpen aria-hidden="true" />
          <span>データベースを開く</span>
        </SidebarMenuButton>
      </SidebarMenuItem>
      {info ? (
        <SidebarMenuItem>
          <SidebarMenuButton tooltip="再読み込み" onClick={() => void reload()}>
            <RefreshCw aria-hidden="true" />
            <span>再読み込み</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      ) : null}
    </SidebarMenu>
  );
}
