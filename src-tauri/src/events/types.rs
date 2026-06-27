// Перенесено из Exsul (src-tauri/src/events/types.rs), сокращено до товарного
// домена. Структуры — это типизированный контракт между Rust-бэкендом и
// фронтендом (serde camelCase где это удобно фронту, snake_case в БД).

use serde::{Deserialize, Serialize};

/// A single row of the append-only event ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: Option<i64>,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub hlc_timestamp: String,
    pub node_id: String,
    pub version: i64,
    pub created_at: Option<String>,
}

/// Materialized product/card row (the `items` projection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub category_id: Option<String>,
    pub initial_price: f64,
    pub current_price: f64,
    pub production_cost: f64,
    pub current_stock: i64,
    pub sold_count: i64,
    pub revenue: f64,
    /// Free-form structured attributes (color/material/size/...) as raw JSON.
    pub attributes_json: String,
    pub image_path: Option<String>,
    pub card_color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Generic paginated list result. `truncated` signals the soft cap was hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPage<T> {
    pub rows: Vec<T>,
    pub truncated: bool,
}

// ── Command payloads ───────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemPayload {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub category_id: Option<String>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub production_cost: Option<f64>,
    #[serde(default)]
    pub initial_stock: Option<i64>,
    /// Optional structured attributes; stored verbatim as JSON.
    #[serde(default)]
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateItemPayload {
    pub item_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub category_id: Option<String>,
    #[serde(default)]
    pub production_cost: Option<f64>,
    #[serde(default)]
    pub attributes: Option<serde_json::Value>,
    #[serde(default)]
    pub card_color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordSalePayload {
    pub item_id: String,
    pub quantity: i64,
    #[serde(default)]
    pub sale_price: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdjustStockPayload {
    pub item_id: String,
    pub delta: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePricePayload {
    pub item_id: String,
    pub new_price: f64,
}

// ── Categories ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCategoryPayload {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

// ── Audit ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuditLogFilter {
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

// ── AI metadata ────────────────────────────────────────────────────

/// Per-item AI metadata (decoded form of the `ai_item_metadata` row).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiItemMetadata {
    pub item_id: String,
    #[serde(default)]
    pub image_caption_ru: Option<String>,
    #[serde(default)]
    pub image_caption_ka: Option<String>,
    #[serde(default)]
    pub image_caption_en: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub visual_attributes: serde_json::Value,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub embedding_model: Option<String>,
    #[serde(default)]
    pub embedding_updated_at: Option<String>,
    #[serde(default)]
    pub ai_updated_at: Option<String>,
}

// ── Analytics (бизнес-аналитика поверх проекции items) ─────────────

#[derive(Debug, Clone, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

/// Image bytes fetched from disk for frontend display. Returned as MIME +
/// base64 so the frontend can build a `data:` URL without needing the asset
/// protocol (which the CSP also allows, but data: URLs keep the request
/// fully self-contained).
#[derive(Debug, Clone, Serialize)]
pub struct ItemImage {
    pub mime: String,
    pub base64: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InventorySummary {
    pub item_count: i64,
    pub total_stock_units: i64,
    /// Σ current_stock * current_price
    pub stock_value_at_price: f64,
    /// Σ current_stock * production_cost
    pub stock_value_at_cost: f64,
    pub total_revenue: f64,
    pub total_sold: i64,
    /// Items with current_stock <= low_stock_threshold.
    pub low_stock_count: i64,
    pub top_categories: Vec<CategoryCount>,
}

// ── Extended analytics ─────────────────────────────────────────────
// All read-only views over the `items` projection + `events` ledger +
// `ai_item_metadata`. Used by `commands::analytics`.

#[derive(Debug, Clone, Serialize)]
pub struct TopSeller {
    pub id: String,
    pub name: String,
    pub category: String,
    pub sold_count: i64,
    pub revenue: f64,
    pub current_price: f64,
    pub current_stock: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeadStock {
    pub id: String,
    pub name: String,
    pub category: String,
    pub current_stock: i64,
    pub current_price: f64,
    /// current_stock * current_price — capital tied up.
    pub locked_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LowStockItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub current_stock: i64,
    pub current_price: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryBreakdown {
    pub category: String,
    pub item_count: i64,
    pub stock_units: i64,
    pub stock_value: f64,
    pub revenue: f64,
    pub sold_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeseriesPoint {
    /// Bucket label, e.g. "2026-06-27", "2026-W26", "2026-06".
    pub bucket: String,
    pub sales_count: i64,
    pub units_sold: i64,
    pub revenue: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiCoverage {
    pub total_items: i64,
    pub with_caption_ka: i64,
    pub with_tags: i64,
    pub with_aliases: i64,
    pub latest_ai_update: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityEntry {
    pub event_type: String,
    pub item_id: String,
    pub item_name: String,
    /// Human-readable single-line summary (e.g. "გაყიდა 2 ცალი", "+10 მარაგი").
    pub summary: String,
    pub hlc_timestamp: String,
    pub created_at: Option<String>,
}

/// Stock-out forecast: predicts how many days of stock remain at the current
/// sales velocity. Items without enough sales history are excluded.
#[derive(Debug, Clone, Serialize)]
pub struct StockOutForecast {
    pub id: String,
    pub name: String,
    pub category: String,
    pub current_stock: i64,
    pub sold_count: i64,
    /// Units sold per day, computed since the first SaleRecorded event.
    pub velocity_per_day: f64,
    /// Estimated days until `current_stock` reaches 0.
    pub days_until_stockout: f64,
    /// ISO timestamp of the first SaleRecorded event for this item (None if none).
    pub first_sale_at: Option<String>,
    /// Number of days of sales history (max(first_sale_at, today) - first_sale_at).
    pub history_days: f64,
}

/// One cell of the sales heatmap (weekday × hour). Only non-zero cells are
/// emitted; the frontend zero-fills the grid.
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapCell {
    /// 0 = Sunday … 6 = Saturday (matches SQLite strftime('%w')).
    pub weekday: u8,
    /// 0..23 (matches SQLite strftime('%H')).
    pub hour: u8,
    pub revenue: f64,
    pub units: i64,
}
