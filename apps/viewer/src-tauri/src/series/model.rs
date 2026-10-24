use monitor_storage::{PingBucket, SystemBucket};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SeriesResponse {
    pub bucket_ms: i64,
    pub ping: Vec<PingBucket>,
    pub system: Vec<SystemBucket>,
}
