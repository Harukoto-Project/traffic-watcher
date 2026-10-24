use tauri::State;

use super::api;
use super::model::Overview;
use crate::error::CommandError;
use crate::range;
use crate::state::{AppState, run_query};

#[tauri::command]
pub async fn load_overview(
    state: State<'_, AppState>,
    since_ms: i64,
    until_ms: i64,
) -> Result<Overview, CommandError> {
    range::validate(since_ms, until_ms)?;
    run_query(&state, move |storage| {
        api::load(&storage, since_ms, until_ms)
    })
    .await
}
