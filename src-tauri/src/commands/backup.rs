// Backup / restore commands. Тяжёлая I/O идёт в spawn_blocking, чтобы не блокировать
// UI-поток.

use crate::files::backup::{self, RestoreReport};
use std::path::PathBuf;
use tauri::AppHandle;

#[tauri::command]
pub async fn export_backup(handle: AppHandle) -> Result<String, String> {
    let h = handle.clone();
    let path = tokio::task::spawn_blocking(move || backup::create_backup(&h))
        .await
        .map_err(|e| e.to_string())??;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn import_backup(handle: AppHandle, path: String) -> Result<RestoreReport, String> {
    let h = handle.clone();
    tokio::task::spawn_blocking(move || backup::restore_backup(&h, PathBuf::from(path)))
        .await
        .map_err(|e| e.to_string())?
}
