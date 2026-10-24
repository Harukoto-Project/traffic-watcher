import { useMemo } from "react";

import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import { buildSnapshotPingData, type ChartSeries } from "@/lib/chartData";
import type { MonitorEvent, SnapshotInfo } from "@/types/monitor";

interface SnapshotPingCardProps {
  info: SnapshotInfo;
  events: MonitorEvent[];
}

export function SnapshotPingCard({ info, events }: SnapshotPingCardProps) {
  const data = useMemo(() => buildSnapshotPingData(events), [events]);
  const series = useMemo(
    () =>
      data.targets.map((target): ChartSeries => ({
        key: `rtt_${target.key}`,
        label: target.label,
        color: target.color,
        dash: target.dash,
      })),
    [data.targets],
  );

  return (
    <ChartCard title="pingの往復時間" empty={data.rows.length === 0}>
      <TimeSeriesChart
        rows={data.rows}
        series={series}
        sinceMs={info.window_start_ms}
        untilMs={info.window_end_ms}
        markers={[info.trigger_ms]}
        syncId="snapshot"
        connectNulls
        yFormatter={(value) => `${value}ms`}
      />
    </ChartCard>
  );
}
