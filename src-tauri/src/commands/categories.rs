// Category commands.

use crate::db::Database;
use crate::events::types::{Category, CreateCategoryPayload};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_categories(db: State<'_, Database>) -> Result<Vec<Category>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_categories(&conn)
}

#[tauri::command]
pub fn create_category(
    db: State<'_, Database>,
    payload: CreateCategoryPayload,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::insert_category(&conn, &id, &payload)?;
    Ok(id)
}

#[tauri::command]
pub fn delete_category(db: State<'_, Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_category(&conn, &id)
}
