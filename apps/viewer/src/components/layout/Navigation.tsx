import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { navigationGroups, type PageId } from "@/config/navigation";

interface NavigationProps {
  page: PageId;
  onChange: (page: PageId) => void;
}

export function Navigation({ page, onChange }: NavigationProps) {
  const { setOpenMobile } = useSidebar();

  return (
    <>
      {navigationGroups.map((group) => (
        <SidebarGroup key={group.id}>
          <SidebarGroupLabel>{group.label}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu aria-label={group.label}>
              {group.items.map((item) => {
                const active = item.id === page;
                const Icon = item.icon;
                return (
                  <SidebarMenuItem key={item.id}>
                    <SidebarMenuButton
                      isActive={active}
                      tooltip={item.label}
                      aria-current={active ? "page" : undefined}
                      onClick={() => {
                        onChange(item.id);
                        setOpenMobile(false);
                      }}
                    >
                      <Icon aria-hidden="true" />
                      <span>{item.label}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      ))}
    </>
  );
}
