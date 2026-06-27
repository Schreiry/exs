// Минимальный preflight (адаптировано из Exsul, сильно упрощено). Выполняется
// ДО инициализации Tauri, поэтому не имеет доступа к AppHandle — путь к данным
// резолвится через переменные окружения. Цель: поймать «нет прав на запись /
// диск полный» и показать понятную ошибку вместо «окно мигнуло и исчезло».

use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)] // String surfaced via Debug in the fatal-path log
pub enum PreflightError {
    NoWritableDataDir(String),
}

fn data_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("APPDATA"))
        .or_else(|| std::env::var_os("HOME"))?;
    Some(PathBuf::from(base).join("com.exsul.app"))
}

/// Verify we can create the data dir and write a probe file. Anything else
/// (WebView2 presence, etc.) is left to Tauri's own error path + fatal dialog.
pub fn run_preflight() -> Result<(), PreflightError> {
    let Some(dir) = data_dir() else {
        // No known base dir — let Tauri try anyway; not fatal here.
        return Ok(());
    };

    if let Err(e) = std::fs::create_dir_all(&dir) {
        return Err(PreflightError::NoWritableDataDir(format!(
            "cannot create {}: {}",
            dir.display(),
            e
        )));
    }

    let probe = dir.join(".write-probe");
    let write_ok = std::fs::File::create(&probe)
        .and_then(|mut f| f.write_all(b"ok"))
        .is_ok();
    let _ = std::fs::remove_file(&probe);

    if !write_ok {
        return Err(PreflightError::NoWritableDataDir(format!(
            "no write access to {}",
            dir.display()
        )));
    }
    Ok(())
}
