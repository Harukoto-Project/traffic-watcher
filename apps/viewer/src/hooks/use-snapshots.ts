import { listSnapshots } from "@/services/snapshotService";
import type { SnapshotInfo } from "@/types/monitor";

import { useAsyncData, type AsyncState } from "./use-async-data";
import { useDatabase } from "./use-database";

const SNAPSHOT_LIMIT = 1000;

export function useSnapshots(): AsyncState<SnapshotInfo[]> {
  const { revision } = useDatabase();
  return useAsyncData(() => listSnapshots(SNAPSHOT_LIMIT), [revision]);
}
