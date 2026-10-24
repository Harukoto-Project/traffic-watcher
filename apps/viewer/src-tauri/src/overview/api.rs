use monitor_storage::Storage;

use super::model::Overview;

pub fn load(storage: &Storage, since_ms: i64, until_ms: i64) -> anyhow::Result<Overview> {
    Ok(Overview {
        ping: storage.summary_range(since_ms, until_ms)?,
        system: storage.system_summary_range(since_ms, until_ms)?,
        alert_count: storage.alert_count_range(since_ms, until_ms)?,
    })
}
