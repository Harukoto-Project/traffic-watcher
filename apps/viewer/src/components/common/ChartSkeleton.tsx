import { Skeleton } from "@/components/ui/skeleton";

export function ChartSkeleton({ height = 224 }: { height?: number }) {
  return (
    <div className="flex flex-col gap-4">
      <Skeleton className="h-6 w-40" />
      <Skeleton className="w-full" style={{ height }} />
    </div>
  );
}
