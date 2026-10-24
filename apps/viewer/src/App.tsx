import { useState } from "react";

import { EmptyState } from "@/components/common/EmptyState";
import { ErrorState } from "@/components/common/ErrorState";
import { PageHeader } from "@/components/common/PageHeader";
import { StatGridSkeleton } from "@/components/common/StatGridSkeleton";
import { AppShell } from "@/components/layout/AppShell";
import { Button } from "@/components/ui/button";
import { DEFAULT_PAGE, getPageTitle, type PageId } from "@/config/navigation";
import { useDatabase } from "@/hooks/use-database";
import { EventsPage } from "@/pages/EventsPage";
import { HourlyPage } from "@/pages/HourlyPage";
import { OverviewPage } from "@/pages/OverviewPage";
import { SnapshotsPage } from "@/pages/SnapshotsPage";
import { TimelinePage } from "@/pages/TimelinePage";

function PageContent({ page }: { page: PageId }) {
  switch (page) {
    case "overview":
      return <OverviewPage />;
    case "timeline":
      return <TimelinePage />;
    case "snapshots":
      return <SnapshotsPage />;
    case "events":
      return <EventsPage />;
    case "hourly":
      return <HourlyPage />;
  }
}

function App() {
  const { status, error, openWithDialog } = useDatabase();
  const [page, setPage] = useState<PageId>(DEFAULT_PAGE);

  return (
    <AppShell page={page} onPageChange={setPage}>
      {status === "ready" ? null : <PageHeader title={getPageTitle(page)} />}
      {error ? <ErrorState error={error} /> : null}
      {status === "loading" ? <StatGridSkeleton count={6} /> : null}
      {status === "empty" ? (
        <EmptyState
          title="データベースが選択されていません"
          description="監視デーモンが記録したデータベースのファイルを選択してください。"
          action={
            <Button onClick={() => void openWithDialog()}>
              データベースを開く
            </Button>
          }
          className="min-h-64"
        />
      ) : null}
      {status === "ready" ? <PageContent page={page} /> : null}
    </AppShell>
  );
}

export default App;
