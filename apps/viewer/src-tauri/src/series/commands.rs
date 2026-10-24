use tauri::State;

use super::api;
use super::model::SeriesResponse;
use crate::error::CommandError;
use crate::range;
use crate::state::{AppState, run_query};

#[tauri::command]
pub async fn load_series(
    state: State<'_, AppState>,
    since_ms: i64,
    until_ms: i64,
    max_points: i64,
) -> Result<SeriesResponse, CommandError> {
    range::validate(since_ms, until_ms)?;
    run_query(&state, move |storage| {
        api::load(&storage, since_ms, until_ms, max_points)
    })
    .await
}
