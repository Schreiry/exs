// Перенесено из Exsul (src-tauri/src/db/mod.rs) — aggressive auto-heal DB init.
// Удалены доменные вызовы (fixup категорий, seed грузинских городов). Слой
// recovery сохранён без изменений: `init_with_recovery` НИКОГДА не паникует и
// всегда регистрирует рабочую Database в AppHandle.

pub mod migrations;
pub mod queries;
pub mod seed;

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

const DB_FILE: &str = "exsul.db";

/// SQLite connection shared across Tauri commands.
///
/// `Arc<Mutex<Connection>>` so heavy commands can `Arc::clone(&db.conn)` and
/// move the clone into `tokio::task::spawn_blocking` — that frees the tokio
/// runtime to handle other commands while a slow query runs on the blocking
/// pool. Synchronous commands keep working unchanged.
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

/// Coarse health classification after `init_with_recovery`. The frontend reads
/// `local_config.last_recovery_state` and shows a banner for anything != Healthy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state")]
pub enum RecoveryState {
    Healthy,
    Recovered {
        backup_path: String,
        reason: String,
        full_backup_path: Option<String>,
    },
    CatastrophicFresh {
        reason: String,
    },
    PartialMigrations {
        failure_count: usize,
    },
}

fn is_unrecoverable_db_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    [
        "database is malformed",
        "database disk image is malformed",
        "disk image is malformed",
        "not a database",
        "file is not a database",
        "file is encrypted or is not a database",
        "database corruption",
        "unable to open database file",
        "attempt to write a readonly database",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

/// Open the DB and run migrations. Returns (connection, failed-migration-count).
fn open_and_migrate(db_path: &Path) -> Result<(Connection, usize), String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .map_err(|e| e.to_string())?;
    let failures = migrations::run(&conn).map_err(|e| e.to_string())?;
    Ok((conn, failures))
}

/// Fold a stale -wal/-shm (process killed mid-write) back into the main DB.
fn cleanup_orphaned_wal(db_path: &Path) {
    let wal = db_path.with_file_name(format!("{}-wal", DB_FILE));
    let shm = db_path.with_file_name(format!("{}-shm", DB_FILE));

    if !wal.exists() && !shm.exists() {
        return;
    }
    if !db_path.exists() {
        let _ = std::fs::remove_file(&wal);
        let _ = std::fs::remove_file(&shm);
        log::warn!("Removed orphaned WAL/SHM with no main DB");
        return;
    }

    log::info!("Found pre-existing WAL/SHM; attempting checkpoint");
    if let Ok(c) = Connection::open(db_path) {
        if let Err(e) = c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);") {
            log::warn!("WAL checkpoint failed (continuing): {}", e);
        }
    }
}

/// Move a corrupted DB (and WAL/SHM siblings) aside, preserving the bytes.
fn quarantine_corrupted_db(db_path: &Path) -> std::io::Result<PathBuf> {
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let dir = db_path.parent().unwrap_or_else(|| Path::new("."));
    let target = dir.join(format!("{DB_FILE}.corrupted-{stamp}.bak"));

    std::fs::rename(db_path, &target)?;

    for ext in ["-wal", "-shm"] {
        let sibling = dir.join(format!("{DB_FILE}{ext}"));
        if sibling.exists() {
            let sibling_target = dir.join(format!("{DB_FILE}{ext}.corrupted-{stamp}.bak"));
            let _ = std::fs::rename(&sibling, &sibling_target);
        }
    }

    write_recovery_log(dir, &format!("DB quarantined to {}", target.display()));
    Ok(target)
}

/// Last-resort: move/delete every exsul.db* file to make room for a fresh DB.
fn nuke_all_db_files(dir: &Path) {
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.starts_with(DB_FILE) {
                    let target = dir.join(format!("{}.broken-{}.bak", name, stamp));
                    if std::fs::rename(&p, &target).is_err() {
                        let _ = std::fs::remove_file(&p);
                    }
                }
            }
        }
    }
    write_recovery_log(dir, "All exsul.db* files removed (catastrophic fresh)");
}

/// Best-effort full backup of the data folder before we touch anything.
fn backup_data_folder(app_dir: &Path) -> Option<PathBuf> {
    let parent = app_dir.parent()?;
    let folder_name = app_dir.file_name()?.to_str()?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let target = parent.join(format!("{}.backup-{}", folder_name, stamp));

    match copy_dir_recursive(app_dir, &target) {
        Ok(()) => {
            log::info!("Full backup created at {:?}", target);
            Some(target)
        }
        Err(e) => {
            log::warn!("Full backup of {:?} failed (continuing): {}", app_dir, e);
            None
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn write_recovery_log(dir: &Path, msg: &str) {
    let recovery_log = dir.join("recovery.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&recovery_log) {
        use std::io::Write;
        let _ = writeln!(f, "[{}] {}", chrono::Utc::now().to_rfc3339(), msg);
    }
}

fn record_recovery_state(conn: &Connection, state: &RecoveryState) {
    let json = serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string());
    let _ = conn.execute(
        "INSERT OR REPLACE INTO local_config (key, value) VALUES ('last_recovery_state', ?1)",
        [&json],
    );
}

/// Initialise the DB with aggressive auto-heal. NEVER returns Err / panics.
///
/// Recovery chain:
///   1. open as-is
///   2. on unrecoverable error: backup folder, quarantine .db files, open fresh
///   3. on second failure: nuke every exsul.db* file, open fresh
///   4. if that fails too: in-memory DB (app launches, warns loudly)
pub fn init_with_recovery(handle: &AppHandle) -> RecoveryState {
    let app_dir = match handle.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            log::error!("app_data_dir() failed ({}); falling back to %APPDATA%", e);
            std::env::var_os("APPDATA")
                .map(|p| std::path::PathBuf::from(p).join("com.exsul.app"))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        }
    };
    let _ = std::fs::create_dir_all(&app_dir);
    let db_path = app_dir.join(DB_FILE);

    cleanup_orphaned_wal(&db_path);

    let (conn, recovery_state, failures) = match open_and_migrate(&db_path) {
        Ok((c, fails)) => {
            let state = if fails == 0 {
                RecoveryState::Healthy
            } else {
                RecoveryState::PartialMigrations { failure_count: fails }
            };
            (c, state, fails)
        }
        Err(first_err) => {
            log::warn!("DB open/migrate failed on attempt 1: {}", first_err);
            if !is_unrecoverable_db_error(&first_err) {
                log::warn!("Non-corruption error treated as corruption for auto-heal");
            }

            let full_backup = backup_data_folder(&app_dir);

            match quarantine_corrupted_db(&db_path) {
                Ok(backup) => {
                    log::warn!("DB quarantined to {:?}; opening fresh", backup);
                    match open_and_migrate(&db_path) {
                        Ok((c, fails)) => {
                            let state = RecoveryState::Recovered {
                                backup_path: backup.display().to_string(),
                                reason: first_err.clone(),
                                full_backup_path: full_backup.as_ref().map(|p| p.display().to_string()),
                            };
                            (c, state, fails)
                        }
                        Err(second_err) => {
                            log::error!("Fresh DB also failed after quarantine: {}", second_err);
                            attempt_3_catastrophic(&app_dir, &db_path, &second_err)
                        }
                    }
                }
                Err(qerr) => {
                    log::error!("Quarantine itself failed: {}; nuking", qerr);
                    attempt_3_catastrophic(&app_dir, &db_path, &qerr.to_string())
                }
            }
        }
    };

    ensure_node_id(&conn);
    record_recovery_state(&conn, &recovery_state);

    if failures > 0 {
        log::warn!("DB initialized with {} failed migration(s)", failures);
    }
    log::info!("Database initialized at {:?}", db_path);
    handle.manage(Database {
        conn: Arc::new(Mutex::new(conn)),
    });

    recovery_state
}

fn attempt_3_catastrophic(
    app_dir: &Path,
    db_path: &Path,
    prior_err: &str,
) -> (Connection, RecoveryState, usize) {
    nuke_all_db_files(app_dir);
    match open_and_migrate(db_path) {
        Ok((c, fails)) => (
            c,
            RecoveryState::CatastrophicFresh { reason: prior_err.to_string() },
            fails,
        ),
        Err(third_err) => {
            log::error!("Catastrophic fresh failed too ({}); using in-memory DB", third_err);
            let conn = Connection::open_in_memory().expect("open_in_memory must succeed");
            let _ = conn.execute_batch("PRAGMA journal_mode = MEMORY; PRAGMA foreign_keys = ON;");
            let fails = migrations::run(&conn).unwrap_or(0);
            (
                conn,
                RecoveryState::CatastrophicFresh {
                    reason: format!("disk DB unusable: {}; running in-memory", third_err),
                },
                fails,
            )
        }
    }
}

fn ensure_node_id(conn: &Connection) {
    let node_id: Option<String> = conn
        .query_row("SELECT value FROM local_config WHERE key = 'node_id'", [], |row| row.get(0))
        .ok();

    if node_id.is_none() {
        let id = uuid::Uuid::new_v4().to_string();
        if let Err(e) = conn.execute(
            "INSERT INTO local_config (key, value) VALUES ('node_id', ?1)",
            [&id],
        ) {
            log::warn!("Could not insert node_id: {}", e);
        } else {
            log::info!("Generated node_id: {}", id);
        }
    }
}

#[allow(dead_code)]
pub fn get_db_path(handle: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_dir.join(DB_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn unique_tmp_dir(label: &str) -> PathBuf {
        let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S-%f").to_string();
        let dir = env::temp_dir().join(format!("exsul-test-{}-{}", label, stamp));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn is_unrecoverable_db_error_recognises_known_markers() {
        for marker in &["database is malformed", "not a database", "unable to open database file"] {
            assert!(is_unrecoverable_db_error(marker));
        }
    }

    #[test]
    fn is_unrecoverable_db_error_ignores_benign_errors() {
        assert!(!is_unrecoverable_db_error("table foo already exists"));
        assert!(!is_unrecoverable_db_error("FOREIGN KEY constraint failed"));
    }

    #[test]
    fn open_and_migrate_creates_schema() {
        let dir = unique_tmp_dir("open-migrate");
        let db_path = dir.join(DB_FILE);
        let (conn, failures) = open_and_migrate(&db_path).expect("must succeed");
        assert_eq!(failures, 0);
        for table in &["events", "items", "ai_item_metadata", "item_search_fts"] {
            let n: i64 = conn
                .query_row("SELECT COUNT(*) FROM sqlite_master WHERE name=?1", [table], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 1, "table '{}' must exist", table);
        }
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn quarantine_renames_db_with_timestamp() {
        let dir = unique_tmp_dir("quarantine");
        let db_path = dir.join(DB_FILE);
        std::fs::write(&db_path, b"fake-corrupted-bytes").unwrap();
        let backup = quarantine_corrupted_db(&db_path).expect("quarantine ok");
        assert!(!db_path.exists());
        assert!(backup.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
