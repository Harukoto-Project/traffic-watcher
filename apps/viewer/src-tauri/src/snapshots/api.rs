use monitor_core::model::SnapshotInfo;
use monitor_storage::Storage;

use super::model::SnapshotDetail;

pub fn list(storage: &Storage, limit: usize) -> anyhow::Result<Vec<SnapshotInfo>> {
    storage.list_snapshots(limit)
}

pub fn load(storage: &Storage, id: i64) -> anyhow::Result<Option<SnapshotDetail>> {
    Ok(storage
        .get_snapshot(id)?
        .map(|(info, events)| SnapshotDetail { info, events }))
}
