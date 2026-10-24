import type { ReactNode } from "react";
import { createPortal } from "react-dom";

import { TimeRangePicker } from "@/components/range/TimeRangePicker";
import { usePageHeaderSlot } from "@/hooks/use-page-header-slot";

interface PageHeaderProps {
  title: string;
  showRange?: boolean;
  children?: ReactNode;
}

export function PageHeader({
  title,
  showRange = false,
  children,
}: PageHeaderProps) {
  const slot = usePageHeaderSlot();

  if (!slot) {
    return null;
  }

  return createPortal(
    <div className="flex w-full flex-wrap items-center gap-x-8 gap-y-3">
      <h1 className="text-xl font-semibold whitespace-nowrap">{title}</h1>
      {showRange ? <TimeRangePicker /> : null}
      {children ? (
        <div className="ml-auto flex max-w-full flex-wrap items-center justify-end gap-x-6 gap-y-3">
          {children}
        </div>
      ) : null}
    </div>,
    slot,
  );
}
