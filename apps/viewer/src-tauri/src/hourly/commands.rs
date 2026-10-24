use monitor_storage::HourlyRow;
use tauri::State;

use super::api;
use crate::error::CommandError;
use crate::range;
use crate::state::{AppState, run_query};

#[tauri::command]
pub async fn list_hourly(
    state: State<'_, AppState>,
    since_ms: i64,
    until_ms: i64,
    source: Option<String>,
) -> Result<Vec<HourlyRow>, CommandError> {
    range::validate(since_ms, until_ms)?;
    run_query(&state, move |storage| {
        api::list(&storage, since_ms, until_ms, source.as_deref())
    })
    .await
}
