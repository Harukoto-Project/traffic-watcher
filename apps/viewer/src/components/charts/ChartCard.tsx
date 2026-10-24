import type { ReactNode } from "react";

import { EmptyState } from "@/components/common/EmptyState";
import { Section } from "@/components/common/Section";

interface ChartCardProps {
  title: string;
  action?: ReactNode;
  empty?: boolean;
  children: ReactNode;
}

export function ChartCard({
  title,
  action,
  empty = false,
  children,
}: ChartCardProps) {
  return (
    <Section title={title} action={action}>
      {empty ? (
        <EmptyState
          title="この期間のデータはありません"
          description="期間を広げて、もう一度確認してください。"
          className="min-h-56"
        />
      ) : (
        children
      )}
    </Section>
  );
}
