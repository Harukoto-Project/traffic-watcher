import { Skeleton } from "@/components/ui/skeleton";

import { SnapshotDetailSkeleton } from "./SnapshotDetailSkeleton";

export function SnapshotsSkeleton() {
  return (
    <div className="grid items-start gap-8 lg:grid-cols-[18rem_minmax(0,1fr)]">
      <div className="flex flex-col gap-1">
        {Array.from({ length: 8 }, (_, index) => (
          <Skeleton key={index} className="h-24 w-full" />
        ))}
      </div>
      <SnapshotDetailSkeleton />
    </div>
  );
}
