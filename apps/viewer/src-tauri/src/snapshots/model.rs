use monitor_core::Event;
use monitor_core::model::SnapshotInfo;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotDetail {
    pub info: SnapshotInfo,
    pub events: Vec<Event>,
}
