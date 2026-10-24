import type { SnapshotDetail, SnapshotInfo } from "@/types/monitor";

import { invokeCommand } from "./invokeCommand";

export function listSnapshots(limit: number): Promise<SnapshotInfo[]> {
  return invokeCommand<SnapshotInfo[]>("list_snapshots", { limit });
}

export function loadSnapshot(id: number): Promise<SnapshotDetail> {
  return invokeCommand<SnapshotDetail>("load_snapshot", { id });
}
