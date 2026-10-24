import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface SectionProps {
  title: string;
  action?: ReactNode;
  className?: string;
  children: ReactNode;
}

export function Section({ title, action, className, children }: SectionProps) {
  return (
    <section className={cn("flex flex-col gap-4", className)}>
      <div className="flex min-h-6 flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <h2 className="text-base font-semibold">{title}</h2>
        {action}
      </div>
      {children}
    </section>
  );
}
