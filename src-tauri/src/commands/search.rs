// Search commands. Локальный поиск по товарам (FTS5 + AI-метаданные).

use crate::db::Database;
use crate::search::{self, ProductSearchResponse};
use tauri::State;

#[tauri::command]
pub fn search_products(
    db: State<'_, Database>,
    query: String,
    limit: Option<u32>,
) -> Result<ProductSearchResponse, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    search::search_products(&conn, &query, limit)
}

/// Rebuild the entire FTS index from items + AI metadata. Returns indexed count.
#[tauri::command]
pub fn rebuild_search_index(db: State<'_, Database>) -> Result<usize, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    search::rebuild_search_index(&conn)
}
