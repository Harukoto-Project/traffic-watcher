import { loadSnapshot } from "@/services/snapshotService";
import type { SnapshotDetail } from "@/types/monitor";

import { useAsyncData, type AsyncState } from "./use-async-data";
import { useDatabase } from "./use-database";

export function useSnapshotDetail(id: number): AsyncState<SnapshotDetail> {
  const { revision } = useDatabase();
  return useAsyncData(() => loadSnapshot(id), [id, revision]);
}
