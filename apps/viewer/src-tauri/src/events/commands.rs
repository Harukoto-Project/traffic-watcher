use monitor_core::Event;
use tauri::State;

use super::api;
use crate::error::CommandError;
use crate::range;
use crate::state::{AppState, run_query};

#[tauri::command]
pub async fn list_events(
    state: State<'_, AppState>,
    since_ms: i64,
    until_ms: i64,
    source: Option<String>,
    kind: Option<String>,
    limit: usize,
) -> Result<Vec<Event>, CommandError> {
    range::validate(since_ms, until_ms)?;
    let kind = api::normalize_kind(kind)?;
    run_query(&state, move |storage| {
        api::list(
            &storage,
            since_ms,
            until_ms,
            source.as_deref(),
            kind.as_deref(),
            limit,
        )
    })
    .await
}
