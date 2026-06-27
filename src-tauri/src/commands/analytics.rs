// Analytics commands — read-only views over items / events / ai_item_metadata.
// Thin wrappers around db::queries that return serde-shaped structs. No
// business logic here — computation stays in queries.rs where it can be unit
// tested in-process.

use crate::db::Database;
use crate::events::types::{
    ActivityEntry, AiCoverage, CategoryBreakdown, DeadStock, HeatmapCell, LowStockItem,
    StockOutForecast, TimeseriesPoint, TopSeller,
};
use tauri::State;

const DEFAULT_LIMIT: i64 = 10;
const MAX_LIMIT: i64 = 100;
const DEFAULT_LOW_STOCK: i64 = 3;
const DEFAULT_MIN_SALES: i64 = 3;

#[tauri::command]
pub fn get_top_sellers(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> Result<Vec<TopSeller>, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_top_sellers(&conn, cap)
}

#[tauri::command]
pub fn get_dead_stock(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> Result<Vec<DeadStock>, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_dead_stock(&conn, cap)
}

#[tauri::command]
pub fn get_low_stock_items(
    db: State<'_, Database>,
    threshold: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<LowStockItem>, String> {
    let thr = threshold.unwrap_or(DEFAULT_LOW_STOCK).max(0);
    let cap = limit.unwrap_or(DEFAULT_LIMIT * 2).clamp(1, MAX_LIMIT);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_low_stock_items(&conn, thr, cap)
}

#[tauri::command]
pub fn get_category_breakdown(db: State<'_, Database>) -> Result<Vec<CategoryBreakdown>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_category_breakdown(&conn)
}

/// `bucket` is one of "day" | "week" | "month" (default "day"). `since` is an
/// optional ISO-8601 lower bound on the event timestamp (e.g. "2026-01-01").
#[tauri::command]
pub fn get_sales_timeseries(
    db: State<'_, Database>,
    bucket: Option<String>,
    since: Option<String>,
) -> Result<Vec<TimeseriesPoint>, String> {
    let b = bucket.unwrap_or_else(|| "day".to_string());
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_sales_timeseries(&conn, &b, since.as_deref())
}

#[tauri::command]
pub fn get_ai_coverage(db: State<'_, Database>) -> Result<AiCoverage, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_ai_coverage(&conn)
}

#[tauri::command]
pub fn get_recent_activity(
    db: State<'_, Database>,
    limit: Option<i64>,
) -> Result<Vec<ActivityEntry>, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_recent_activity(&conn, cap)
}

/// Predict stock-out risk. Items with fewer than `min_sales` SaleRecorded
/// events are excluded (velocity can't be trusted on a single sale).
#[tauri::command]
pub fn get_stock_out_forecast(
    db: State<'_, Database>,
    limit: Option<i64>,
    min_sales: Option<i64>,
) -> Result<Vec<StockOutForecast>, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let ms = min_sales.unwrap_or(DEFAULT_MIN_SALES).max(1);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_stock_out_forecast(&conn, cap, ms)
}

/// Sales heatmap (weekday × hour). `since` is an optional ISO-8601 lower
/// bound on the event timestamp (e.g. "2026-01-01").
#[tauri::command]
pub fn get_sales_heatmap(
    db: State<'_, Database>,
    since: Option<String>,
) -> Result<Vec<HeatmapCell>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_sales_heatmap(&conn, since.as_deref())
}