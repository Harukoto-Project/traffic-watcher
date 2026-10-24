import type { ReactNode } from "react";

import { STATUS_STYLES, type StatusLevel } from "@/config/status";
import { cn } from "@/lib/utils";

interface StatusTextProps {
  level: StatusLevel;
  className?: string;
  children: ReactNode;
}

export function StatusText({ level, className, children }: StatusTextProps) {
  const style = STATUS_STYLES[level];
  const Icon = style.icon;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5",
        style.textClass,
        className,
      )}
    >
      {Icon ? <Icon aria-hidden="true" className="size-3.5 shrink-0" /> : null}
      {children}
    </span>
  );
}
