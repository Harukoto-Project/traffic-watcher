use monitor_core::model::SnapshotInfo;
use tauri::State;

use super::api;
use super::model::SnapshotDetail;
use crate::error::CommandError;
use crate::state::{AppState, run_query};

const MAX_SNAPSHOT_LIMIT: usize = 2000;

#[tauri::command]
pub async fn list_snapshots(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<SnapshotInfo>, CommandError> {
    let limit = limit.clamp(1, MAX_SNAPSHOT_LIMIT);
    run_query(&state, move |storage| api::list(&storage, limit)).await
}

#[tauri::command]
pub async fn load_snapshot(
    state: State<'_, AppState>,
    id: i64,
) -> Result<SnapshotDetail, CommandError> {
    let detail = run_query(&state, move |storage| api::load(&storage, id)).await?;
    detail.ok_or_else(|| {
        CommandError::new(
            "snapshot_not_found",
            format!("スナップショット#{id}が見つかりません"),
        )
    })
}
