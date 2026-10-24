import { useState } from "react";

import { AsyncView } from "@/components/common/AsyncView";
import { PageHeader } from "@/components/common/PageHeader";
import { SnapshotDetailView } from "@/components/snapshots/SnapshotDetailView";
import { SnapshotList } from "@/components/snapshots/SnapshotList";
import { SnapshotsSkeleton } from "@/components/snapshots/SnapshotsSkeleton";
import { getPageTitle } from "@/config/navigation";
import { useSnapshots } from "@/hooks/use-snapshots";

export function SnapshotsPage() {
  const snapshots = useSnapshots();
  const [pickedId, setPickedId] = useState<number | null>(null);

  return (
    <>
      <PageHeader title={getPageTitle("snapshots")} />
      <AsyncView state={snapshots} skeleton={<SnapshotsSkeleton />}>
        {(list) => {
          const selectedId = list.some((item) => item.id === pickedId)
            ? pickedId
            : (list[0]?.id ?? null);
          return (
            <div className="grid items-start gap-8 lg:grid-cols-[18rem_minmax(0,1fr)]">
              <SnapshotList
                snapshots={list}
                selectedId={selectedId}
                onSelect={setPickedId}
              />
              {selectedId !== null ? (
                <div className="flex min-w-0 flex-col gap-10">
                  <SnapshotDetailView key={selectedId} id={selectedId} />
                </div>
              ) : null}
            </div>
          );
        }}
      </AsyncView>
    </>
  );
}
