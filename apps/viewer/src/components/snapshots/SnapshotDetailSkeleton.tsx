import { ChartSkeleton } from "@/components/common/ChartSkeleton";
import { TableSkeleton } from "@/components/common/TableSkeleton";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

export function SnapshotDetailSkeleton() {
  return (
    <div className="flex flex-col gap-10">
      <Card>
        <CardHeader>
          <Skeleton className="h-6 w-48" />
          <Skeleton className="h-5 w-72 max-w-full" />
        </CardHeader>
        <CardContent className="grid grid-cols-2 gap-x-6 gap-y-4 xl:grid-cols-4">
          {Array.from({ length: 4 }, (_, index) => (
            <div key={index} className="flex flex-col gap-1">
              <Skeleton className="h-5 w-16" />
              <Skeleton className="h-5 w-28" />
            </div>
          ))}
        </CardContent>
      </Card>
      <ChartSkeleton height={224} />
      <ChartSkeleton height={180} />
      <TableSkeleton rows={5} withHeading />
    </div>
  );
}
