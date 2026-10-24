use std::path::PathBuf;

use tauri::State;

use super::api;
use super::model::DatabaseInfo;
use crate::error::CommandError;
use crate::state::{AppState, run_blocking, run_query};

#[tauri::command]
pub async fn open_database(
    state: State<'_, AppState>,
    path: String,
) -> Result<DatabaseInfo, CommandError> {
    let path = PathBuf::from(path);
    let target = path.clone();
    let (storage, info) = run_blocking(move || api::open(&target)).await?;
    state.set(path, storage);
    Ok(info)
}

#[tauri::command]
pub fn close_database(state: State<'_, AppState>) {
    state.clear();
}

#[tauri::command]
pub async fn get_database_info(
    state: State<'_, AppState>,
) -> Result<Option<DatabaseInfo>, CommandError> {
    let Some(path) = state.path() else {
        return Ok(None);
    };
    let info = run_query(&state, move |storage| api::describe(&path, &storage)).await?;
    Ok(Some(info))
}

#[tauri::command]
pub fn find_default_database() -> Option<String> {
    api::find_default().map(|path| path.display().to_string())
}
