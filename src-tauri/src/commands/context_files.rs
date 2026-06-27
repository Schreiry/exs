//! Typed Tauri commands for explicitly selected local AI-context files.

use crate::files::local_context::{
    read_registered_file, ContextFileAccess, ContextFileDocument, ContextFileError,
    ContextFileErrorCode, ContextFileIssue, ContextFileSelection, ALLOWED_CONTEXT_EXTENSIONS,
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn select_context_files(
    app: AppHandle,
    access: State<'_, ContextFileAccess>,
) -> Result<ContextFileSelection, ContextFileError> {
    // The plugin's blocking picker must not run on the async runtime worker.
    let picked = tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Choose files for AI context")
            .add_filter("Text and structured files", ALLOWED_CONTEXT_EXTENSIONS)
            .blocking_pick_files()
    })
    .await
    .map_err(|_| {
        ContextFileError::new(
            ContextFileErrorCode::DialogFailed,
            "The system file picker could not be opened.",
            None,
        )
    })?;

    let Some(picked) = picked else {
        return Ok(ContextFileSelection::cancelled());
    };

    let mut paths = Vec::with_capacity(picked.len());
    let mut rejected = Vec::new();
    for selected in picked {
        match selected.into_path() {
            Ok(path) => paths.push(path),
            Err(_) => rejected.push(ContextFileIssue {
                code: ContextFileErrorCode::UnsupportedPath,
                message: "This platform-specific file location cannot be read safely.".to_string(),
                file_name: "unknown".to_string(),
            }),
        }
    }
    access.register_paths(paths, rejected)
}

#[tauri::command]
pub async fn read_context_file(
    access: State<'_, ContextFileAccess>,
    selection_id: String,
) -> Result<ContextFileDocument, ContextFileError> {
    // Resolve and clone the registered path before awaiting I/O, so the mutex
    // is never held while the disk is being read.
    let registered = access.resolve(&selection_id)?;
    read_registered_file(registered).await
}

#[tauri::command]
pub fn forget_context_file(
    access: State<'_, ContextFileAccess>,
    selection_id: String,
) -> Result<bool, ContextFileError> {
    access.forget(&selection_id)
}

#[tauri::command]
pub fn clear_context_files(
    access: State<'_, ContextFileAccess>,
) -> Result<usize, ContextFileError> {
    access.clear()
}
