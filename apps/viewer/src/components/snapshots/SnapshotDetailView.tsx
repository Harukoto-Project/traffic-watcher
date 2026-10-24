import { AsyncView } from "@/components/common/AsyncView";
import { useSnapshotDetail } from "@/hooks/use-snapshot-detail";

import { ProcessRankingCard } from "./ProcessRankingCard";
import { SnapshotDetailSkeleton } from "./SnapshotDetailSkeleton";
import { SnapshotEventsCard } from "./SnapshotEventsCard";
import { SnapshotHeaderCard } from "./SnapshotHeaderCard";
import { SnapshotPingCard } from "./SnapshotPingCard";
import { SnapshotSystemCard } from "./SnapshotSystemCard";

export function SnapshotDetailView({ id }: { id: number }) {
  const detail = useSnapshotDetail(id);

  return (
    <AsyncView state={detail} skeleton={<SnapshotDetailSkeleton />}>
      {(data) => (
        <>
          <SnapshotHeaderCard info={data.info} />
          <SnapshotPingCard info={data.info} events={data.events} />
          <SnapshotSystemCard info={data.info} events={data.events} />
          <ProcessRankingCard
            events={data.events}
            triggerMs={data.info.trigger_ms}
          />
          <SnapshotEventsCard events={data.events} />
        </>
      )}
    </AsyncView>
  );
}
