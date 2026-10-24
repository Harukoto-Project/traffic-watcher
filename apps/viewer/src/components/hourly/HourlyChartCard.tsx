import { useMemo } from "react";

import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import type { HourlyMetric } from "@/config/hourlyMetrics";
import { buildHourlyChartData } from "@/lib/chartData";
import type { HourlyRow } from "@/types/monitor";

interface HourlyChartCardProps {
  rows: HourlyRow[];
  metric: HourlyMetric;
  sinceMs: number;
  untilMs: number;
}

export function HourlyChartCard({
  rows,
  metric,
  sinceMs,
  untilMs,
}: HourlyChartCardProps) {
  const chart = useMemo(
    () => buildHourlyChartData(rows, metric),
    [rows, metric],
  );

  return (
    <ChartCard title={metric.label} empty={chart.rows.length === 0}>
      <TimeSeriesChart
        rows={chart.rows}
        series={chart.series}
        sinceMs={sinceMs}
        untilMs={untilMs}
        yMax={metric.fixedMax}
        yFormatter={(value) => `${value}${metric.unit}`}
        height={320}
      />
    </ChartCard>
  );
}
