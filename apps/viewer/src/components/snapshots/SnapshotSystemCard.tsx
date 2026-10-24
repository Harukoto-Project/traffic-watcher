import { useMemo } from "react";

import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import {
  buildSnapshotSystemData,
  chartColor,
  chartDash,
  type ChartSeries,
} from "@/lib/chartData";
import type { MonitorEvent, SnapshotInfo } from "@/types/monitor";

const SERIES: ChartSeries[] = [
  { key: "cpu", label: "CPU", color: chartColor(0) },
  { key: "mem", label: "メモリ", color: chartColor(1), dash: chartDash(1) },
];

interface SnapshotSystemCardProps {
  info: SnapshotInfo;
  events: MonitorEvent[];
}

export function SnapshotSystemCard({ info, events }: SnapshotSystemCardProps) {
  const rows = useMemo(() => buildSnapshotSystemData(events), [events]);

  return (
    <ChartCard title="CPUとメモリの使用率" empty={rows.length === 0}>
      <TimeSeriesChart
        rows={rows}
        series={SERIES}
        sinceMs={info.window_start_ms}
        untilMs={info.window_end_ms}
        markers={[info.trigger_ms]}
        syncId="snapshot"
        yMax={100}
        yFormatter={(value) => `${value}%`}
      />
    </ChartCard>
  );
}
