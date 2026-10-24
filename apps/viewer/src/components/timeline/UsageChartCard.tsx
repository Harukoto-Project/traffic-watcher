import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import {
  chartColor,
  chartDash,
  MAX_DASH,
  type ChartRow,
  type ChartSeries,
} from "@/lib/chartData";

interface UsageChartCardProps {
  rows: ChartRow[];
  sinceMs: number;
  untilMs: number;
  markers: number[];
  syncId: string;
}

const SERIES: ChartSeries[] = [
  { key: "cpu_avg", label: "CPU平均", color: chartColor(0) },
  { key: "cpu_max", label: "CPU最大", color: chartColor(0), dash: MAX_DASH },
  {
    key: "mem_avg",
    label: "メモリ",
    color: chartColor(1),
    dash: chartDash(1),
  },
];

export function UsageChartCard({
  rows,
  sinceMs,
  untilMs,
  markers,
  syncId,
}: UsageChartCardProps) {
  return (
    <ChartCard title="CPUとメモリ" empty={rows.length === 0}>
      <TimeSeriesChart
        rows={rows}
        series={SERIES}
        sinceMs={sinceMs}
        untilMs={untilMs}
        markers={markers}
        syncId={syncId}
        yMax={100}
        yFormatter={(value) => `${value}%`}
        height={180}
      />
    </ChartCard>
  );
}
