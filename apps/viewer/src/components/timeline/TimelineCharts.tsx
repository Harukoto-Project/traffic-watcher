import { useMemo } from "react";

import { buildPingChartData, buildSystemChartData } from "@/lib/chartData";
import type { SeriesResponse } from "@/types/monitor";
import type { TimeRange } from "@/types/timeRange";

import { LossChartCard } from "./LossChartCard";
import { NetworkChartCard } from "./NetworkChartCard";
import { PingChartCard } from "./PingChartCard";
import { UsageChartCard } from "./UsageChartCard";

interface TimelineChartsProps {
  data: SeriesResponse;
  range: TimeRange;
  markers: number[];
}

const SYNC_ID = "timeline";

export function TimelineCharts({ data, range, markers }: TimelineChartsProps) {
  const ping = useMemo(
    () => buildPingChartData(data.ping, data.bucket_ms),
    [data],
  );
  const system = useMemo(
    () => buildSystemChartData(data.system, data.bucket_ms),
    [data],
  );
  const common = {
    sinceMs: range.sinceMs,
    untilMs: range.untilMs,
    markers,
    syncId: SYNC_ID,
  };

  return (
    <>
      <PingChartCard data={ping} {...common} />
      <LossChartCard data={ping} {...common} />
      <UsageChartCard rows={system} {...common} />
      <NetworkChartCard rows={system} {...common} />
    </>
  );
}
