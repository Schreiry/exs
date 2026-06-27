// Перенесено из Exsul (src-tauri/src/events/store.rs), сокращено.
// Append-only event writes + audit log. Проекция обновляется триггерами
// (миграция 002), поэтому здесь только запись в ledger.

use crate::db::Database;
use crate::events::types::EventRecord;
use crate::sync::hlc::HybridLogicalClock;
use rusqlite::{params, Connection};

/// Append an event (locks the DB mutex internally).
pub fn append_event(
    db: &Database,
    hlc: &HybridLogicalClock,
    aggregate_id: &str,
    aggregate_type: &str,
    event_type: &str,
    data: serde_json::Value,
) -> Result<EventRecord, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    append_event_with_conn(&conn, hlc, aggregate_id, aggregate_type, event_type, data)
}

/// Append an event using an already-held connection — required when the caller
/// is inside a wider transaction so the event and its side effects stay atomic.
pub fn append_event_with_conn(
    conn: &Connection,
    hlc: &HybridLogicalClock,
    aggregate_id: &str,
    aggregate_type: &str,
    event_type: &str,
    data: serde_json::Value,
) -> Result<EventRecord, String> {
    let node_id = hlc.node_id().to_string();
    let hlc_timestamp = hlc.now();
    let version = crate::db::queries::get_next_version(conn, aggregate_id, &node_id)?;
    let data_str = serde_json::to_string(&data).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO events (aggregate_id, aggregate_type, event_type, data, hlc_timestamp, node_id, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![aggregate_id, aggregate_type, event_type, data_str, hlc_timestamp, node_id, version],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    Ok(EventRecord {
        id: Some(id),
        aggregate_id: aggregate_id.to_string(),
        aggregate_type: aggregate_type.to_string(),
        event_type: event_type.to_string(),
        data,
        hlc_timestamp,
        node_id,
        version,
        created_at: None,
    })
}

/// Append a human-readable audit record. Non-fatal: callers log-and-continue.
/// Секреты НИКОГДА не пишутся в audit_logs.
pub fn append_audit_log(
    db: &Database,
    user_id: &str,
    action: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::insert_audit_log(&conn, user_id, action, &payload)
}
