// File storage layer — product images + backup/restore.
// Адаптировано из Exsul (save_item_image + sync/backup), но упрощено:
// без шифрования/Drive. Backup = zip(БД + images) через online-backup API.

pub mod backup;
pub mod local_context;

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Resolve (and create) the app images directory: <app_data>/images.
pub fn images_dir(handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let dir = app_dir.join("images");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Write decoded image bytes to <app_data>/images/<item_id>.img and return the
/// relative path stored on the item. File I/O happens BEFORE the DB lock.
pub async fn write_item_image(
    handle: &AppHandle,
    item_id: &str,
    bytes: &[u8],
) -> Result<String, String> {
    let dir = images_dir(handle)?;
    let file_name = format!("{}.img", item_id);
    let file_path = dir.join(&file_name);
    tokio::fs::write(&file_path, bytes)
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("images/{}", file_name))
}

/// Sniff image MIME from magic bytes. We store images without an extension
/// (single `.img` blob per item), so we need this at read time to build the
/// correct `data:` URL for the frontend.
pub fn sniff_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else {
        "image/jpeg"
    }
}
