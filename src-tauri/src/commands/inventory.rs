// Inventory commands — товары/карточки. Адаптировано из Exsul (commands/inventory.rs):
// запись через события (event-sourced) + audit log; проекция items обновляется
// триггерами. После записи переиндексируем FTS, чтобы поиск был актуален.

use crate::db::Database;
use crate::events::store;
use crate::events::types::*;
use crate::search;
use crate::sync::hlc::HybridLogicalClock;
use serde_json::json;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

#[tauri::command]
pub fn add_item(
    db: State<'_, Database>,
    hlc: State<'_, HybridLogicalClock>,
    payload: CreateItemPayload,
) -> Result<String, String> {
    let item_id = Uuid::new_v4().to_string();
    let attributes_json = payload
        .attributes
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let data = json!({
        "name": payload.name,
        "description": payload.description.clone().unwrap_or_default(),
        "category": payload.category.clone().unwrap_or_else(|| "uncategorized".to_string()),
        "category_id": payload.category_id,
        "price": payload.price.unwrap_or(0.0),
        "production_cost": payload.production_cost.unwrap_or(0.0),
        "initial_stock": payload.initial_stock.unwrap_or(0),
        "attributes_json": attributes_json,
    });

    store::append_event(&db, &hlc, &item_id, "item", "ItemCreated", data.clone())?;

    // Index for search + audit (both non-fatal beyond the event itself).
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = search::reindex_item_fts(&conn, &item_id);
    }
    if let Err(e) = store::append_audit_log(&db, "local", "ItemCreated", data) {
        log::warn!("audit log write failed: {}", e);
    }

    Ok(item_id)
}

#[tauri::command]
pub fn update_item(
    db: State<'_, Database>,
    hlc: State<'_, HybridLogicalClock>,
    payload: UpdateItemPayload,
) -> Result<(), String> {
    // card_color is pure UI metadata — write directly, not via events.
    if let Some(ref color) = payload.card_color {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::queries::update_item_card_color(
            &conn,
            &payload.item_id,
            if color.is_empty() {
                None
            } else {
                Some(color.as_str())
            },
        )?;
    }

    let has_business_change = payload.name.is_some()
        || payload.description.is_some()
        || payload.category.is_some()
        || payload.category_id.is_some()
        || payload.production_cost.is_some()
        || payload.attributes.is_some();

    if has_business_change {
        let data = json!({
            "name": payload.name,
            "description": payload.description,
            "category": payload.category,
            "category_id": payload.category_id,
            "production_cost": payload.production_cost,
            "attributes_json": payload.attributes.as_ref().map(|v| v.to_string()),
        });
        store::append_event(
            &db,
            &hlc,
            &payload.item_id,
            "item",
            "ItemUpdated",
            data.clone(),
        )?;
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = search::reindex_item_fts(&conn, &payload.item_id);
        }
        if let Err(e) = store::append_audit_log(&db, "local", "ItemUpdated", data) {
            log::warn!("audit log write failed: {}", e);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_items(db: State<'_, Database>, limit: Option<u32>) -> Result<ListPage<Item>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_items(&conn, limit)
}

#[tauri::command]
pub fn get_item(db: State<'_, Database>, item_id: String) -> Result<Option<Item>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_item_by_id(&conn, &item_id)
}

#[tauri::command]
pub fn record_sale(
    db: State<'_, Database>,
    hlc: State<'_, HybridLogicalClock>,
    payload: RecordSalePayload,
) -> Result<(), String> {
    let data = json!({ "quantity": payload.quantity, "sale_price": payload.sale_price });
    store::append_event(
        &db,
        &hlc,
        &payload.item_id,
        "item",
        "SaleRecorded",
        data.clone(),
    )?;
    let _ = store::append_audit_log(&db, "local", "SaleRecorded", data);
    Ok(())
}

#[tauri::command]
pub fn adjust_stock(
    db: State<'_, Database>,
    hlc: State<'_, HybridLogicalClock>,
    payload: AdjustStockPayload,
) -> Result<(), String> {
    let data = json!({ "delta": payload.delta });
    store::append_event(
        &db,
        &hlc,
        &payload.item_id,
        "item",
        "StockAdjusted",
        data.clone(),
    )?;
    let _ = store::append_audit_log(&db, "local", "StockAdjusted", data);
    Ok(())
}

#[tauri::command]
pub fn change_price(
    db: State<'_, Database>,
    hlc: State<'_, HybridLogicalClock>,
    payload: ChangePricePayload,
) -> Result<(), String> {
    let data = json!({ "new_price": payload.new_price });
    store::append_event(
        &db,
        &hlc,
        &payload.item_id,
        "item",
        "PriceChanged",
        data.clone(),
    )?;
    let _ = store::append_audit_log(&db, "local", "PriceChanged", data);
    Ok(())
}

#[tauri::command]
pub fn delete_item(db: State<'_, Database>, item_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_item(&conn, &item_id)
}

#[tauri::command]
pub fn delete_all_items(db: State<'_, Database>) -> Result<usize, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_all_items(&conn)
}

#[tauri::command]
pub fn duplicate_item(db: State<'_, Database>, item_id: String) -> Result<String, String> {
    let new_id = Uuid::new_v4().to_string();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::duplicate_item(&conn, &item_id, &new_id)?;
    let _ = search::reindex_item_fts(&conn, &new_id);
    Ok(new_id)
}

/// Saves a base64-encoded image to disk then records the path on the item.
/// File I/O happens BEFORE locking the DB mutex (ported from Exsul).
#[tauri::command]
pub async fn save_item_image(
    handle: AppHandle,
    db: State<'_, Database>,
    item_id: String,
    base64_data: String,
) -> Result<String, String> {
    if base64_data.len() > 7 * 1024 * 1024 {
        return Err("Image exceeds 5 MB limit".to_string());
    }
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let bytes = STANDARD
        .decode(&base64_data)
        .map_err(|e| format!("base64 decode error: {}", e))?;

    let relative_path = crate::files::write_item_image(&handle, &item_id, &bytes).await?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::set_item_image_path(&conn, &item_id, &relative_path)?;
    drop(conn);

    let _ = store::append_audit_log(
        &db,
        "local",
        "ItemImageSaved",
        json!({ "item_id": item_id, "path": relative_path }),
    );
    Ok(relative_path)
}

/// Load an item's image bytes for the frontend. Returns None when the item
/// has no image attached. Used by the void interface to render a hero image
/// above (and inside) product cards when search results match.
#[tauri::command]
pub fn get_item_image(
    handle: AppHandle,
    db: State<'_, Database>,
    item_id: String,
) -> Result<Option<ItemImage>, String> {
    // Resolve the relative path from the DB (no file I/O under lock).
    let rel: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::queries::get_item_by_id(&conn, &item_id)?.and_then(|i| i.image_path)
    };
    let Some(rel) = rel else {
        return Ok(None);
    };

    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let abs = app_dir.join(&rel);
    let bytes = std::fs::read(&abs).map_err(|e| format!("read image: {e}"))?;
    let mime = crate::files::sniff_mime(&bytes).to_string();

    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let base64 = STANDARD.encode(&bytes);
    Ok(Some(ItemImage { mime, base64 }))
}

/// Inventory analytics summary (item count, stock value, low-stock, top categories).
#[tauri::command]
pub fn get_inventory_summary(
    db: State<'_, Database>,
    low_stock_threshold: Option<i64>,
) -> Result<InventorySummary, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_inventory_summary(&conn, low_stock_threshold.unwrap_or(3))
}

/// Insert demo items (no-op if items already exist). Returns inserted count.
#[tauri::command]
pub fn seed_demo_items(db: State<'_, Database>) -> Result<usize, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::seed::seed_demo_items(&conn)
}
