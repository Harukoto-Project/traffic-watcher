use monitor_core::model::SourceSummary;
use monitor_storage::SystemSummary;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub ping: Vec<SourceSummary>,
    pub system: SystemSummary,
    pub alert_count: i64,
}
