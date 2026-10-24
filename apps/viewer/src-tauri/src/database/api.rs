use std::path::{Path, PathBuf};

use monitor_storage::Storage;

use super::model::DatabaseInfo;
use crate::error::CommandError;

const SEARCH_DEPTH: usize = 6;
const DATABASE_FILE: &str = "monitor.db";

pub fn describe(path: &Path, storage: &Storage) -> anyhow::Result<DatabaseInfo> {
    let size_bytes = std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
    Ok(DatabaseInfo {
        path: path.display().to_string(),
        size_bytes,
        stats: storage.database_stats()?,
    })
}

pub fn open(path: &Path) -> Result<(Storage, DatabaseInfo), CommandError> {
    if !path.is_file() {
        return Err(CommandError::new(
            "database_not_found",
            format!("ファイルが見つかりません: {}", path.display()),
        ));
    }
    let storage = Storage::open_readonly(path).map_err(|err| {
        CommandError::new(
            "database_open_failed",
            format!("データベースを開けませんでした: {err:#}"),
        )
    })?;
    let valid = storage.has_expected_schema().unwrap_or(false);
    if !valid {
        return Err(CommandError::new(
            "invalid_database",
            "traffic-watcherのデータベースではありません",
        ));
    }
    let info = describe(path, &storage).map_err(CommandError::from)?;
    Ok((storage, info))
}

fn candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        roots.push(dir);
    }
    for root in roots {
        for dir in root.ancestors().take(SEARCH_DEPTH) {
            out.push(dir.join("data").join(DATABASE_FILE));
        }
    }
    if let Some(program_data) = std::env::var_os("ProgramData") {
        out.push(
            PathBuf::from(program_data)
                .join("monitor")
                .join(DATABASE_FILE),
        );
    }
    out
}

pub fn find_default() -> Option<PathBuf> {
    candidates().into_iter().find(|path| path.is_file())
}
