import { useMemo } from "react";

import { AsyncView } from "@/components/common/AsyncView";
import { PageHeader } from "@/components/common/PageHeader";
import { TimelineCharts } from "@/components/timeline/TimelineCharts";
import { TimelineSkeleton } from "@/components/timeline/TimelineSkeleton";
import { getPageTitle } from "@/config/navigation";
import { useSeries } from "@/hooks/use-series";
import { useSnapshots } from "@/hooks/use-snapshots";
import { useTimeRange } from "@/hooks/use-time-range";

export function TimelinePage() {
  const series = useSeries();
  const snapshots = useSnapshots();
  const { range } = useTimeRange();

  const markers = useMemo(
    () =>
      (snapshots.data ?? [])
        .map((snapshot) => snapshot.trigger_ms)
        .filter((ts) => ts >= range.sinceMs && ts <= range.untilMs),
    [snapshots.data, range.sinceMs, range.untilMs],
  );

  return (
    <>
      <PageHeader title={getPageTitle("timeline")} showRange />
      <AsyncView state={series} skeleton={<TimelineSkeleton />}>
        {(data) => (
          <TimelineCharts data={data} range={range} markers={markers} />
        )}
      </AsyncView>
    </>
  );
}
