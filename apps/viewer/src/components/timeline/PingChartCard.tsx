import { useMemo, useState } from "react";

import { ChartCard } from "@/components/charts/ChartCard";
import { TimeSeriesChart } from "@/components/charts/TimeSeriesChart";
import { Checkbox } from "@/components/ui/checkbox";
import {
  MAX_DASH,
  type ChartSeries,
  type PingChartData,
} from "@/lib/chartData";

interface PingChartCardProps {
  data: PingChartData;
  sinceMs: number;
  untilMs: number;
  markers: number[];
  syncId: string;
}

export function PingChartCard({
  data,
  sinceMs,
  untilMs,
  markers,
  syncId,
}: PingChartCardProps) {
  const [showMax, setShowMax] = useState(false);

  const series = useMemo(
    () =>
      data.targets.flatMap((target) => {
        const list: ChartSeries[] = [
          {
            key: `avg_${target.key}`,
            label: target.label,
            color: target.color,
            dash: target.dash,
          },
        ];
        if (showMax) {
          list.push({
            key: `max_${target.key}`,
            label: `${target.label}(最大)`,
            color: target.color,
            dash: MAX_DASH,
          });
        }
        return list;
      }),
    [data.targets, showMax],
  );

  return (
    <ChartCard
      title="pingの推移"
      empty={data.rows.length === 0}
      action={
        <label className="flex cursor-pointer items-center gap-2 text-sm">
          <Checkbox
            checked={showMax}
            onCheckedChange={(checked) => setShowMax(checked === true)}
          />
          最大値を表示
        </label>
      }
    >
      <TimeSeriesChart
        rows={data.rows}
        series={series}
        sinceMs={sinceMs}
        untilMs={untilMs}
        markers={markers}
        syncId={syncId}
        yFormatter={(value) => `${value}ms`}
      />
    </ChartCard>
  );
}
