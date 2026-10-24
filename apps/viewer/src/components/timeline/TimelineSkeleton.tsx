import { ChartSkeleton } from "@/components/common/ChartSkeleton";

export function TimelineSkeleton() {
  return (
    <div className="flex flex-col gap-10">
      <ChartSkeleton height={224} />
      <ChartSkeleton height={160} />
      <ChartSkeleton height={180} />
      <ChartSkeleton height={180} />
    </div>
  );
}
