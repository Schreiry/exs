// Audit / event-log read commands.

use crate::db::Database;
use crate::events::types::{AuditLog, AuditLogFilter, EventRecord};
use tauri::State;

#[tauri::command]
pub fn get_audit_logs(
    db: State<'_, Database>,
    filter: Option<AuditLogFilter>,
) -> Result<Vec<AuditLog>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_audit_logs(&conn, &filter.unwrap_or_default())
}

#[tauri::command]
pub fn get_events(db: State<'_, Database>, limit: Option<i64>) -> Result<Vec<EventRecord>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_events(&conn, limit)
}
