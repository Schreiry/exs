// System/status commands — версия, состояние recovery, состояние БД.

use crate::db::Database;
use serde_json::Value;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_app_version(handle: AppHandle) -> String {
    handle.package_info().version.to_string()
}

/// Recovery state persisted by db::init_with_recovery (banner source for the UI).
#[tauri::command]
pub fn get_recovery_state(db: State<'_, Database>) -> Result<Value, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM local_config WHERE key = 'last_recovery_state'",
            [],
            |r| r.get(0),
        )
        .ok();
    match raw {
        Some(s) => serde_json::from_str(&s).map_err(|e| e.to_string()),
        None => Ok(serde_json::json!({ "state": "Healthy" })),
    }
}
