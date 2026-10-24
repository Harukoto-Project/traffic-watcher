import { useContext } from "react";

import { PageHeaderSlotContext } from "@/contexts/pageHeaderSlotContext";

export function usePageHeaderSlot() {
  return useContext(PageHeaderSlotContext);
}
