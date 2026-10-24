import { StatGridSkeleton } from "@/components/common/StatGridSkeleton";
import { TableSkeleton } from "@/components/common/TableSkeleton";

export function OverviewSkeleton() {
  return (
    <div className="flex flex-col gap-10">
      <StatGridSkeleton count={6} />
      <TableSkeleton rows={3} withHeading />
    </div>
  );
}
