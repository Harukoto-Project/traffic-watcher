import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import {
  chartColor,
  chartDash,
  type ChartRow,
  type ChartSeries,
} from "@/lib/chartData";

interface NetworkChartCardProps {
  rows: ChartRow[];
  sinceMs: number;
  untilMs: number;
  markers: number[];
  syncId: string;
}

const SERIES: ChartSeries[] = [
  { key: "rx_avg", label: "受信", color: chartColor(0) },
  { key: "tx_avg", label: "送信", color: chartColor(1), dash: chartDash(1) },
];

export function NetworkChartCard({
  rows,
  sinceMs,
  untilMs,
  markers,
  syncId,
}: NetworkChartCardProps) {
  return (
    <ChartCard title="ネットワーク" empty={rows.length === 0}>
      <TimeSeriesChart
        rows={rows}
        series={SERIES}
        sinceMs={sinceMs}
        untilMs={untilMs}
        markers={markers}
        syncId={syncId}
        yFormatter={(value) => `${value}MB/s`}
        height={180}
      />
    </ChartCard>
  );
}
