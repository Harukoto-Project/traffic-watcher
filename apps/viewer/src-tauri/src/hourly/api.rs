use monitor_storage::{HourlyRow, Storage};

pub fn list(
    storage: &Storage,
    since_ms: i64,
    until_ms: i64,
    source: Option<&str>,
) -> anyhow::Result<Vec<HourlyRow>> {
    let rows = storage.query_hourly(since_ms, source.filter(|s| !s.is_empty()))?;
    Ok(rows
        .into_iter()
        .filter(|row| row.hour_ms <= until_ms)
        .collect())
}
