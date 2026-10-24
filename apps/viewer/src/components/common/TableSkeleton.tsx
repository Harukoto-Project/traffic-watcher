import { Skeleton } from "@/components/ui/skeleton";

interface TableSkeletonProps {
  rows?: number;
  withHeading?: boolean;
}

export function TableSkeleton({
  rows = 6,
  withHeading = false,
}: TableSkeletonProps) {
  return (
    <div className="flex flex-col gap-4">
      {withHeading ? <Skeleton className="h-6 w-40" /> : null}
      <div className="flex flex-col gap-1">
        <Skeleton className="h-9 w-full" />
        {Array.from({ length: rows }, (_, index) => (
          <Skeleton key={index} className="h-9 w-full" />
        ))}
      </div>
    </div>
  );
}
