import { AsyncView } from "@/components/common/AsyncView";
import { PageHeader } from "@/components/common/PageHeader";
import { DatabaseSummaryCard } from "@/components/overview/DatabaseSummaryCard";
import { OverviewSkeleton } from "@/components/overview/OverviewSkeleton";
import { PingSummaryTable } from "@/components/overview/PingSummaryTable";
import { SystemSummaryGrid } from "@/components/overview/SystemSummaryGrid";
import { getPageTitle } from "@/config/navigation";
import { useOverview } from "@/hooks/use-overview";

export function OverviewPage() {
  const overview = useOverview();

  return (
    <>
      <PageHeader title={getPageTitle("overview")} showRange />
      <AsyncView state={overview} skeleton={<OverviewSkeleton />}>
        {(data) => (
          <>
            <SystemSummaryGrid
              system={data.system}
              alertCount={data.alert_count}
            />
            <PingSummaryTable summaries={data.ping} />
          </>
        )}
      </AsyncView>
      <DatabaseSummaryCard />
    </>
  );
}
