use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use monitor_storage::Storage;

use crate::error::CommandError;

struct OpenDatabase {
    path: PathBuf,
    storage: Storage,
}

#[derive(Default)]
pub struct AppState {
    current: Mutex<Option<OpenDatabase>>,
}

impl AppState {
    fn lock(&self) -> MutexGuard<'_, Option<OpenDatabase>> {
        self.current.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set(&self, path: PathBuf, storage: Storage) {
        *self.lock() = Some(OpenDatabase { path, storage });
    }

    pub fn clear(&self) {
        *self.lock() = None;
    }

    pub fn storage(&self) -> Result<Storage, CommandError> {
        self.lock()
            .as_ref()
            .map(|db| db.storage.clone())
            .ok_or_else(CommandError::database_not_open)
    }

    pub fn path(&self) -> Option<PathBuf> {
        self.lock().as_ref().map(|db| db.path.clone())
    }
}

pub async fn run_blocking<T, F>(job: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, CommandError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|err| CommandError::new("task_failed", err.to_string()))?
}

pub async fn run_query<T, F>(state: &AppState, job: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce(Storage) -> anyhow::Result<T> + Send + 'static,
{
    let storage = state.storage()?;
    run_blocking(move || job(storage).map_err(CommandError::from)).await
}
