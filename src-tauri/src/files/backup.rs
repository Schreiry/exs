// Backup / restore. Адаптировано из Exsul (sync/backup.rs), но без шифрования
// и облака — это опциональные модули более поздней фазы. Формат: zip-архив
// с консистентным снимком БД (online-backup API rusqlite) + папкой images/.

use rusqlite::DatabaseName;
use serde::Serialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, Serialize)]
pub struct RestoreReport {
    pub items_restored: i64,
    pub images_restored: usize,
    pub message: String,
}

fn stamp() -> String {
    chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string()
}

/// Create a zip backup of the live DB + images. Blocking; callers should run
/// this on a blocking task. Returns the absolute path of the .zip.
pub fn create_backup(handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let backups_dir = app_dir.join("backups");
    std::fs::create_dir_all(&backups_dir).map_err(|e| e.to_string())?;

    let s = stamp();
    let tmp_db = std::env::temp_dir().join(format!("exsul-snapshot-{s}.db"));

    // 1. Consistent snapshot of the live DB into a temp file (safe while open).
    {
        let db = handle.state::<crate::db::Database>();
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        conn.backup(DatabaseName::Main, &tmp_db, None)
            .map_err(|e| format!("snapshot failed: {e}"))?;
    }

    // 2. Zip the snapshot + images directory.
    let zip_path = backups_dir.join(format!("exsul-backup-{s}.zip"));
    let file = File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("exsul.db", opts).map_err(|e| e.to_string())?;
    let mut db_bytes = Vec::new();
    File::open(&tmp_db)
        .and_then(|mut f| f.read_to_end(&mut db_bytes))
        .map_err(|e| e.to_string())?;
    zip.write_all(&db_bytes).map_err(|e| e.to_string())?;

    let images = app_dir.join("images");
    if images.is_dir() {
        add_dir_to_zip(&mut zip, &images, "images", opts)?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&tmp_db);
    log::info!("Backup written to {:?}", zip_path);
    Ok(zip_path)
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<File>,
    dir: &Path,
    prefix: &str,
    opts: SimpleFileOptions,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let zip_name = format!("{prefix}/{name}");
        if path.is_file() {
            zip.start_file(&zip_name, opts).map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            File::open(&path)
                .and_then(|mut f| f.read_to_end(&mut bytes))
                .map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Restore from a zip backup into the LIVE database (rusqlite restore API) and
/// copy images back. Blocking. The DB is overwritten in place — no restart
/// required, the projection tables are part of the snapshot.
pub fn restore_backup(handle: &AppHandle, zip_path: PathBuf) -> Result<RestoreReport, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let tmp_dir = std::env::temp_dir().join(format!("exsul-restore-{}", stamp()));
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;

    // Extract.
    let file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("not a valid backup: {e}"))?;
    archive.extract(&tmp_dir).map_err(|e| e.to_string())?;

    let src_db = tmp_dir.join("exsul.db");
    if !src_db.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err("backup archive does not contain exsul.db".to_string());
    }

    // Restore DB into the live connection.
    {
        let db = handle.state::<crate::db::Database>();
        let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.restore(DatabaseName::Main, &src_db, None::<fn(rusqlite::backup::Progress)>)
            .map_err(|e| format!("restore failed: {e}"))?;
    }

    // Copy images back.
    let mut images_restored = 0usize;
    let src_images = tmp_dir.join("images");
    if src_images.is_dir() {
        let dst_images = app_dir.join("images");
        std::fs::create_dir_all(&dst_images).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(&src_images).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().is_file() {
                let dst = dst_images.join(entry.file_name());
                std::fs::copy(entry.path(), dst).map_err(|e| e.to_string())?;
                images_restored += 1;
            }
        }
    }

    let items_restored: i64 = {
        let db = handle.state::<crate::db::Database>();
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0)).unwrap_or(0)
    };

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(RestoreReport {
        items_restored,
        images_restored,
        message: format!("აღდგენილია {items_restored} ჩანაწერი და {images_restored} სურათი"),
    })
}
