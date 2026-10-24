use monitor_storage::DatabaseStats;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DatabaseInfo {
    pub path: String,
    pub size_bytes: u64,
    #[serde(flatten)]
    pub stats: DatabaseStats,
}
