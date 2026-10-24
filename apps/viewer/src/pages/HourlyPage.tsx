import { useState } from "react";

import { AsyncView } from "@/components/common/AsyncView";
import { ChartSkeleton } from "@/components/common/ChartSkeleton";
import { PageHeader } from "@/components/common/PageHeader";
import { HourlyChartCard } from "@/components/hourly/HourlyChartCard";
import { HourlyMetricToggle } from "@/components/hourly/HourlyMetricToggle";
import {
  DEFAULT_HOURLY_METRIC_ID,
  hourlyMetrics,
} from "@/config/hourlyMetrics";
import { getPageTitle } from "@/config/navigation";
import { useHourly } from "@/hooks/use-hourly";
import { useTimeRange } from "@/hooks/use-time-range";

export function HourlyPage() {
  const [metricId, setMetricId] = useState(DEFAULT_HOURLY_METRIC_ID);
  const hourly = useHourly();
  const { range } = useTimeRange();
  const metric =
    hourlyMetrics.find((item) => item.id === metricId) ?? hourlyMetrics[0];

  return (
    <>
      <PageHeader title={getPageTitle("hourly")} showRange>
        <HourlyMetricToggle value={metricId} onChange={setMetricId} />
      </PageHeader>
      <AsyncView state={hourly} skeleton={<ChartSkeleton height={320} />}>
        {(rows) => (
          <HourlyChartCard
            rows={rows}
            metric={metric}
            sinceMs={range.sinceMs}
            untilMs={range.untilMs}
          />
        )}
      </AsyncView>
    </>
  );
}
