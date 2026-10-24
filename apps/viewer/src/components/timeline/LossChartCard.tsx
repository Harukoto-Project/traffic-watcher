import { useMemo } from "react";

import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import { STATUS_STYLES } from "@/config/status";
import type { ChartSeries, PingChartData } from "@/lib/chartData";

interface LossChartCardProps {
  data: PingChartData;
  sinceMs: number;
  untilMs: number;
  markers: number[];
  syncId: string;
}

export function LossChartCard({
  data,
  sinceMs,
  untilMs,
  markers,
  syncId,
}: LossChartCardProps) {
  const series = useMemo<ChartSeries[]>(
    () =>
      data.targets.map((target) => ({
        key: `lost_${target.key}`,
        label: target.label,
        color: STATUS_STYLES.critical.color,
        dash: target.dash,
      })),
    [data.targets],
  );

  return (
    <ChartCard title="パケットロス" empty={data.rows.length === 0}>
      <TimeSeriesChart
        rows={data.rows}
        series={series}
        sinceMs={sinceMs}
        untilMs={untilMs}
        markers={markers}
        syncId={syncId}
        variant="area"
        stepped
        height={160}
      />
    </ChartCard>
  );
}
