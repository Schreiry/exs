// Адаптировано из Exsul (src-tauri/src/db/queries.rs). Оставлено только то,
// что нужно ядру AI-ассистента: товары, категории, audit, события, AI-метаданные
// и бизнес-аналитика. Тяжёлый доменный SQL (orders/flowers/...) не переносился.

use crate::events::types::{
    ActivityEntry, AiCoverage, AiItemMetadata, AuditLog, AuditLogFilter, Category, CategoryBreakdown,
    CategoryCount, CreateCategoryPayload, DeadStock, EventRecord, HeatmapCell, InventorySummary,
    Item, ListPage, LowStockItem, StockOutForecast, TimeseriesPoint, TopSeller,
};
use rusqlite::{params, Connection};

const DEFAULT_ITEMS_LIMIT: u32 = 500;

const ITEM_COLUMNS: &str = "id, name, description, category, category_id, initial_price, \
    current_price, production_cost, current_stock, sold_count, revenue, attributes_json, \
    image_path, card_color, created_at, updated_at";

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<Item> {
    Ok(Item {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        category: row.get(3)?,
        category_id: row.get(4)?,
        initial_price: row.get(5)?,
        current_price: row.get(6)?,
        production_cost: row.get(7)?,
        current_stock: row.get(8)?,
        sold_count: row.get(9)?,
        revenue: row.get(10)?,
        attributes_json: row.get(11)?,
        image_path: row.get(12)?,
        card_color: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

// ============================================================
// Local config
// ============================================================

pub fn get_node_id(conn: &Connection) -> Result<String, String> {
    conn.query_row(
        "SELECT value FROM local_config WHERE key = 'node_id'",
        [],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

// ============================================================
// Items
// ============================================================

pub fn get_items(conn: &Connection, limit: Option<u32>) -> Result<ListPage<Item>, String> {
    let cap = limit.unwrap_or(DEFAULT_ITEMS_LIMIT);
    let sql = format!("SELECT {ITEM_COLUMNS} FROM items ORDER BY updated_at DESC LIMIT ?1");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    // Fetch cap+1 to detect truncation in one query; drop the extra row.
    let probe = (cap as i64) + 1;
    let mut items = stmt
        .query_map([probe], row_to_item)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let truncated = items.len() > cap as usize;
    if truncated {
        items.truncate(cap as usize);
    }
    Ok(ListPage { rows: items, truncated })
}

pub fn get_item_by_id(conn: &Connection, item_id: &str) -> Result<Option<Item>, String> {
    let sql = format!("SELECT {ITEM_COLUMNS} FROM items WHERE id = ?1");
    match conn.query_row(&sql, [item_id], row_to_item) {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Load items by a set of ids, preserving the input order. Helper for batch
/// hydration (e.g. embedding-rerank path on the ai branch).
#[allow(dead_code)]
pub fn get_items_by_ids(conn: &Connection, ids: &[String]) -> Result<Vec<Item>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(item) = get_item_by_id(conn, id)? {
            out.push(item);
        }
    }
    Ok(out)
}

pub fn set_item_image_path(conn: &Connection, item_id: &str, path: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE items SET image_path = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%f','now') WHERE id = ?2",
        params![path, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_item_card_color(
    conn: &Connection,
    item_id: &str,
    color: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE items SET card_color = ?1 WHERE id = ?2",
        params![color, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_item(conn: &Connection, item_id: &str) -> Result<(), String> {
    // ON DELETE CASCADE clears item_prices, item_photos, ai_item_metadata.
    conn.execute("DELETE FROM items WHERE id = ?1", [item_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM item_search_fts WHERE item_id = ?1", [item_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_all_items(conn: &Connection) -> Result<usize, String> {
    let n = conn.execute("DELETE FROM items", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM item_search_fts", []).map_err(|e| e.to_string())?;
    Ok(n)
}

pub fn duplicate_item(conn: &Connection, item_id: &str, new_id: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO items (id, name, description, category, category_id, initial_price,
            current_price, production_cost, current_stock, sold_count, revenue, attributes_json,
            image_path, card_color)
         SELECT ?1, name || ' (copy)', description, category, category_id, initial_price,
            current_price, production_cost, current_stock, 0, 0.0, attributes_json,
            image_path, card_color
         FROM items WHERE id = ?2",
        params![new_id, item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================
// Events / versioning
// ============================================================

pub fn get_next_version(conn: &Connection, aggregate_id: &str, node_id: &str) -> Result<i64, String> {
    let result: Option<i64> = conn
        .query_row(
            "SELECT MAX(version) FROM events WHERE aggregate_id = ?1 AND node_id = ?2",
            params![aggregate_id, node_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(result.unwrap_or(0) + 1)
}

pub fn get_events(conn: &Connection, limit: Option<i64>) -> Result<Vec<EventRecord>, String> {
    let limit = limit.unwrap_or(500);
    let mut stmt = conn
        .prepare(
            "SELECT id, aggregate_id, aggregate_type, event_type, data, hlc_timestamp, node_id, version, created_at
             FROM events ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([limit], |row| {
            let data_str: String = row.get(4)?;
            Ok(EventRecord {
                id: Some(row.get(0)?),
                aggregate_id: row.get(1)?,
                aggregate_type: row.get(2)?,
                event_type: row.get(3)?,
                data: serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null),
                hlc_timestamp: row.get(5)?,
                node_id: row.get(6)?,
                version: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

// ============================================================
// Categories
// ============================================================

pub fn get_categories(conn: &Connection) -> Result<Vec<Category>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, icon, created_at FROM categories ORDER BY name ASC")
        .map_err(|e| e.to_string())?;
    let cats = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(cats)
}

pub fn insert_category(conn: &Connection, id: &str, payload: &CreateCategoryPayload) -> Result<(), String> {
    conn.execute(
        "INSERT INTO categories (id, name, color, icon) VALUES (?1, ?2, ?3, ?4)",
        params![id, payload.name, payload.color, payload.icon],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_category(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM categories WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================
// AI metadata
// ============================================================

/// Upsert AI metadata for an item. Only the provided fields are written —
/// existing values for omitted fields are preserved (COALESCE on update path).
pub fn upsert_ai_metadata(conn: &Connection, meta: &AiItemMetadata) -> Result<(), String> {
    let tags_json = serde_json::to_string(&meta.tags).map_err(|e| e.to_string())?;
    let aliases_json = serde_json::to_string(&meta.aliases).map_err(|e| e.to_string())?;
    let visual_json = if meta.visual_attributes.is_null() {
        "{}".to_string()
    } else {
        serde_json::to_string(&meta.visual_attributes).map_err(|e| e.to_string())?
    };

    conn.execute(
        "INSERT INTO ai_item_metadata
            (item_id, image_caption_ru, image_caption_ka, image_caption_en,
             tags_json, visual_attributes_json, aliases_json,
             embedding_model, embedding_updated_at, ai_updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%Y-%m-%dT%H:%M:%fZ','now'))
         ON CONFLICT(item_id) DO UPDATE SET
            image_caption_ru = COALESCE(excluded.image_caption_ru, ai_item_metadata.image_caption_ru),
            image_caption_ka = COALESCE(excluded.image_caption_ka, ai_item_metadata.image_caption_ka),
            image_caption_en = COALESCE(excluded.image_caption_en, ai_item_metadata.image_caption_en),
            tags_json = excluded.tags_json,
            visual_attributes_json = excluded.visual_attributes_json,
            aliases_json = excluded.aliases_json,
            embedding_model = COALESCE(excluded.embedding_model, ai_item_metadata.embedding_model),
            embedding_updated_at = COALESCE(excluded.embedding_updated_at, ai_item_metadata.embedding_updated_at),
            ai_updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
        params![
            meta.item_id,
            meta.image_caption_ru,
            meta.image_caption_ka,
            meta.image_caption_en,
            tags_json,
            visual_json,
            aliases_json,
            meta.embedding_model,
            meta.embedding_updated_at,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_ai_metadata(conn: &Connection, item_id: &str) -> Result<Option<AiItemMetadata>, String> {
    let result = conn.query_row(
        "SELECT item_id, image_caption_ru, image_caption_ka, image_caption_en,
                tags_json, visual_attributes_json, aliases_json,
                embedding_model, embedding_updated_at, ai_updated_at
         FROM ai_item_metadata WHERE item_id = ?1",
        [item_id],
        |row| {
            let tags_json: String = row.get(4)?;
            let visual_json: String = row.get(5)?;
            let aliases_json: String = row.get(6)?;
            Ok(AiItemMetadata {
                item_id: row.get(0)?,
                image_caption_ru: row.get(1)?,
                image_caption_ka: row.get(2)?,
                image_caption_en: row.get(3)?,
                tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                visual_attributes: serde_json::from_str(&visual_json).unwrap_or(serde_json::json!({})),
                aliases: serde_json::from_str(&aliases_json).unwrap_or_default(),
                embedding_model: row.get(7)?,
                embedding_updated_at: row.get(8)?,
                ai_updated_at: row.get(9)?,
            })
        },
    );
    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// ============================================================
// Audit logs (секреты НЕ пишутся сюда)
// ============================================================

pub fn insert_audit_log(
    conn: &Connection,
    user_id: &str,
    action: &str,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let payload_str = serde_json::to_string(payload).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO audit_logs (user_id, action, payload) VALUES (?1, ?2, ?3)",
        params![user_id, action, payload_str],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_audit_logs(conn: &Connection, filter: &AuditLogFilter) -> Result<Vec<AuditLog>, String> {
    let limit = filter.limit.unwrap_or(200);
    let mut conditions: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1usize;

    if let Some(ref act) = filter.action {
        conditions.push(format!("action LIKE ?{}", idx));
        params_vec.push(Box::new(format!("%{}%", act)));
        idx += 1;
    }
    if let Some(ref since) = filter.since {
        conditions.push(format!("created_at >= ?{}", idx));
        params_vec.push(Box::new(since.clone()));
        idx += 1;
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT id, user_id, action, payload, created_at FROM audit_logs {where_clause} ORDER BY id DESC LIMIT ?{idx}"
    );
    params_vec.push(Box::new(limit));

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            let payload_str: String = row.get(3)?;
            Ok(AuditLog {
                id: row.get(0)?,
                user_id: row.get(1)?,
                action: row.get(2)?,
                payload: serde_json::from_str(&payload_str).unwrap_or(serde_json::Value::Null),
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

// ============================================================
// Business analytics (ядро аналитики поверх проекции items)
// ============================================================

pub fn get_inventory_summary(conn: &Connection, low_stock_threshold: i64) -> Result<InventorySummary, String> {
    let (item_count, total_stock_units, stock_value_at_price, stock_value_at_cost, total_revenue, total_sold, low_stock_count): (i64, i64, f64, f64, f64, i64, i64) =
        conn.query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(current_stock), 0),
                COALESCE(SUM(current_stock * current_price), 0.0),
                COALESCE(SUM(current_stock * production_cost), 0.0),
                COALESCE(SUM(revenue), 0.0),
                COALESCE(SUM(sold_count), 0),
                COALESCE(SUM(CASE WHEN current_stock <= ?1 THEN 1 ELSE 0 END), 0)
             FROM items",
            [low_stock_threshold],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT category, COUNT(*) c FROM items GROUP BY category ORDER BY c DESC LIMIT 8")
        .map_err(|e| e.to_string())?;
    let top_categories = stmt
        .query_map([], |row| {
            Ok(CategoryCount {
                category: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(InventorySummary {
        item_count,
        total_stock_units,
        stock_value_at_price,
        stock_value_at_cost,
        total_revenue,
        total_sold,
        low_stock_count,
        top_categories,
    })
}

// =============================================================
// Extended analytics — read-only views over items / events / AI.
// All queries are non-mutating; safe to call from any command.
// =============================================================

pub fn get_top_sellers(conn: &Connection, limit: i64) -> Result<Vec<TopSeller>, String> {
    let cap = limit.clamp(1, 100);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, sold_count, revenue, current_price, current_stock
             FROM items
             WHERE sold_count > 0
             ORDER BY sold_count DESC, revenue DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([cap], |row| {
            Ok(TopSeller {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                sold_count: row.get(3)?,
                revenue: row.get(4)?,
                current_price: row.get(5)?,
                current_stock: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_dead_stock(conn: &Connection, limit: i64) -> Result<Vec<DeadStock>, String> {
    let cap = limit.clamp(1, 100);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, current_stock, current_price,
                    (current_stock * current_price) AS locked_value
             FROM items
             WHERE sold_count = 0 AND current_stock > 0
             ORDER BY locked_value DESC, current_stock DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([cap], |row| {
            Ok(DeadStock {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                current_stock: row.get(3)?,
                current_price: row.get(4)?,
                locked_value: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_low_stock_items(
    conn: &Connection,
    threshold: i64,
    limit: i64,
) -> Result<Vec<LowStockItem>, String> {
    let cap = limit.clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, current_stock, current_price
             FROM items
             WHERE current_stock <= ?1
             ORDER BY current_stock ASC, name ASC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![threshold, cap], |row| {
            Ok(LowStockItem {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                current_stock: row.get(3)?,
                current_price: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_category_breakdown(conn: &Connection) -> Result<Vec<CategoryBreakdown>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT category,
                    COUNT(*)                              AS item_count,
                    COALESCE(SUM(current_stock), 0)       AS stock_units,
                    COALESCE(SUM(current_stock * current_price), 0.0) AS stock_value,
                    COALESCE(SUM(revenue), 0.0)           AS revenue,
                    COALESCE(SUM(sold_count), 0)          AS sold_count
             FROM items
             GROUP BY category
             ORDER BY revenue DESC, COUNT(*) DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CategoryBreakdown {
                category: row.get(0)?,
                item_count: row.get(1)?,
                stock_units: row.get(2)?,
                stock_value: row.get(3)?,
                revenue: row.get(4)?,
                sold_count: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// Sales time-series over the events ledger. `bucket` ∈ "day" | "week" | "month".
/// `since` is an optional ISO-8601 lower bound on `created_at`. Returns points in
/// ascending bucket order.
pub fn get_sales_timeseries(
    conn: &Connection,
    bucket: &str,
    since: Option<&str>,
) -> Result<Vec<TimeseriesPoint>, String> {
    let bucket_expr = match bucket {
        "day" => "strftime('%Y-%m-%d', created_at)",
        "week" => "strftime('%Y-W%W', created_at)",
        "month" => "strftime('%Y-%m', created_at)",
        _ => return Err(format!("invalid bucket '{}': expected day|week|month", bucket)),
    };
    let sql = format!(
        "SELECT {bucket_expr} AS bucket,
                COUNT(*),
                COALESCE(SUM(CAST(json_extract(data, '$.quantity') AS INTEGER)), 0),
                COALESCE(SUM(
                    CAST(COALESCE(json_extract(data, '$.sale_price'), 0) AS REAL)
                    * CAST(COALESCE(json_extract(data, '$.quantity'), 1) AS INTEGER)
                ), 0.0)
         FROM events
         WHERE event_type = 'SaleRecorded'
           AND (?1 IS NULL OR created_at >= ?1)
         GROUP BY bucket
         ORDER BY bucket ASC"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![since], |row| {
            Ok(TimeseriesPoint {
                bucket: row.get(0)?,
                sales_count: row.get(1)?,
                units_sold: row.get(2)?,
                revenue: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub fn get_ai_coverage(conn: &Connection) -> Result<AiCoverage, String> {
    let (total, with_ka, with_tags, with_aliases, latest): (i64, i64, i64, i64, Option<String>) =
        conn.query_row(
            "SELECT
                (SELECT COUNT(*) FROM items),
                (SELECT COUNT(*) FROM ai_item_metadata WHERE image_caption_ka IS NOT NULL AND image_caption_ka != ''),
                (SELECT COUNT(*) FROM ai_item_metadata WHERE tags_json != '[]'),
                (SELECT COUNT(*) FROM ai_item_metadata WHERE aliases_json != '[]'),
                (SELECT MAX(ai_updated_at) FROM ai_item_metadata)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|e| e.to_string())?;
    Ok(AiCoverage {
        total_items: total,
        with_caption_ka: with_ka,
        with_tags,
        with_aliases,
        latest_ai_update: latest,
    })
}

/// Latest business events with a human-readable per-row summary. Filters to
/// the event types that matter for an SMB owner (no internal scaffolding).
pub fn get_recent_activity(conn: &Connection, limit: i64) -> Result<Vec<ActivityEntry>, String> {
    let cap = limit.clamp(1, 200);
    let mut stmt = conn
        .prepare(
            "SELECT e.event_type, e.aggregate_id, e.hlc_timestamp, e.created_at, e.data,
                    COALESCE(i.name, '')
             FROM events e
             LEFT JOIN items i ON i.id = e.aggregate_id
             WHERE e.event_type IN ('ItemCreated','SaleRecorded','StockAdjusted','PriceChanged')
             ORDER BY e.id DESC
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([cap], |row| {
            let event_type: String = row.get(0)?;
            let item_id: String = row.get(1)?;
            let hlc_timestamp: String = row.get(2)?;
            let created_at: Option<String> = row.get(3)?;
            let data_str: String = row.get(4)?;
            let item_name: String = row.get(5)?;
            let data: serde_json::Value =
                serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);
            let summary = summarize_event(&event_type, &item_name, &data);
            Ok(ActivityEntry {
                event_type,
                item_id,
                item_name,
                summary,
                hlc_timestamp,
                created_at,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// Build a short, locale-neutral summary string for an event row. The
/// frontend can choose to translate further; this keeps the Rust side free of
/// hard-coded KA strings (mirrors the i18n rule for prompts).
fn summarize_event(event_type: &str, item_name: &str, data: &serde_json::Value) -> String {
    match event_type {
        "ItemCreated" => format!("created: {}", item_name),
        "SaleRecorded" => {
            let qty = data.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1);
            let price = data.get("sale_price").and_then(|v| v.as_f64());
            match price {
                Some(p) => format!("sold {} × {} ({:.2})", qty, item_name, p),
                None => format!("sold {} × {}", qty, item_name),
            }
        }
        "StockAdjusted" => {
            let delta = data.get("delta").and_then(|v| v.as_i64()).unwrap_or(0);
            if delta >= 0 {
                format!("stock +{} ({})", delta, item_name)
            } else {
                format!("stock {} ({})", delta, item_name)
            }
        }
        "PriceChanged" => {
            let price = data.get("new_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            format!("price → {:.2} ({})", price, item_name)
        }
        _ => format!("{}: {}", event_type, item_name),
    }
}

/// Sales heatmap over the events ledger. Aggregates SaleRecorded events into
/// a (weekday 0..6, hour 0..23) grid. Only non-zero cells are emitted; the
/// frontend zero-fills for the grid render. `since_iso` is an optional lower
/// bound on `created_at` (e.g. "2026-01-01" or a full ISO timestamp).
pub fn get_sales_heatmap(
    conn: &Connection,
    since_iso: Option<&str>,
) -> Result<Vec<HeatmapCell>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT CAST(strftime('%w', created_at) AS INTEGER) AS weekday,
                    CAST(strftime('%H', created_at) AS INTEGER) AS hour,
                    COALESCE(SUM(
                        CAST(COALESCE(json_extract(data, '$.sale_price'), 0) AS REAL)
                        * CAST(COALESCE(json_extract(data, '$.quantity'), 1) AS INTEGER)
                    ), 0.0) AS revenue,
                    COALESCE(SUM(CAST(COALESCE(json_extract(data, '$.quantity'), 1) AS INTEGER)), 0) AS units
             FROM events
             WHERE event_type = 'SaleRecorded'
               AND (?1 IS NULL OR created_at >= ?1)
             GROUP BY weekday, hour
             ORDER BY weekday ASC, hour ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![since_iso], |row| {
            let weekday: i64 = row.get(0)?;
            let hour: i64 = row.get(1)?;
            Ok(HeatmapCell {
                weekday: weekday.clamp(0, 6) as u8,
                hour: hour.clamp(0, 23) as u8,
                revenue: row.get(2)?,
                units: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

/// Predict which items will run out of stock soon, based on historical sales
/// velocity. Items must have at least `min_sales` SaleRecorded events AND a
/// positive current_stock to be included (no-point predicting items that
/// already exhausted stock or never sold). Sorted by days-until-stockout ASC
/// (most urgent first).
///
/// Velocity = sold_count / (now - first_sale_at). The "now" reference is the
/// real wall-clock (`datetime('now')`) — this gives a sensible forecast even
/// when the user just started recording sales today.
pub fn get_stock_out_forecast(
    conn: &Connection,
    limit: i64,
    min_sales: i64,
) -> Result<Vec<StockOutForecast>, String> {
    let cap = limit.clamp(1, 100);
    let min_sales = min_sales.max(1);

    let mut stmt = conn
        .prepare(
            "WITH first_sales AS (
                SELECT aggregate_id AS item_id,
                       MIN(hlc_timestamp) AS first_sale_at,
                       COUNT(*) AS sale_events
                FROM events
                WHERE event_type = 'SaleRecorded'
                GROUP BY aggregate_id
            )
            SELECT i.id,
                   i.name,
                   i.category,
                   i.current_stock,
                   i.sold_count,
                   fs.first_sale_at,
                   fs.sale_events,
                   CAST((julianday(datetime('now')) - julianday(fs.first_sale_at)) AS REAL) AS history_days
            FROM items i
            JOIN first_sales fs ON fs.item_id = i.id
            WHERE i.current_stock > 0
              AND fs.sale_events >= ?1
              AND fs.first_sale_at IS NOT NULL
              AND julianday(datetime('now')) > julianday(fs.first_sale_at)
            ORDER BY
                CASE WHEN i.sold_count > 0
                     THEN CAST(i.current_stock AS REAL) * (julianday(datetime('now')) - julianday(fs.first_sale_at))
                          / CAST(i.sold_count AS REAL)
                     ELSE 1e18 END ASC
            LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![min_sales, cap], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let category: String = row.get(2)?;
            let current_stock: i64 = row.get(3)?;
            let sold_count: i64 = row.get(4)?;
            let first_sale_at: Option<String> = row.get(5)?;
            let history_days: f64 = row.get(7)?;

            let velocity_per_day = if history_days > 0.0 && sold_count > 0 {
                sold_count as f64 / history_days
            } else {
                0.0
            };
            let days_until_stockout = if velocity_per_day > 0.0 {
                current_stock as f64 / velocity_per_day
            } else {
                f64::INFINITY
            };

            Ok(StockOutForecast {
                id,
                name,
                category,
                current_stock,
                sold_count,
                velocity_per_day,
                days_until_stockout,
                first_sale_at,
                history_days,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::CreateCategoryPayload;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    fn seed_item(conn: &Connection, id: &str, name: &str, price: f64, stock: i64) {
        conn.execute(
            "INSERT INTO events (aggregate_id, aggregate_type, event_type, data, hlc_timestamp, node_id, version)
             VALUES (?1, 'item', 'ItemCreated', ?2, '0:0:n', 'n', 1)",
            params![id, format!("{{\"name\":\"{name}\",\"price\":{price},\"initial_stock\":{stock}}}")],
        )
        .unwrap();
    }

    #[test]
    fn items_crud_and_summary() {
        let conn = db();
        seed_item(&conn, "a", "ჩაი", 10.0, 5);
        seed_item(&conn, "b", "ყავა", 20.0, 1);

        let page = get_items(&conn, None).unwrap();
        assert_eq!(page.rows.len(), 2);

        let summary = get_inventory_summary(&conn, 2).unwrap();
        assert_eq!(summary.item_count, 2);
        assert_eq!(summary.total_stock_units, 6);
        assert_eq!(summary.low_stock_count, 1, "only 'b' (stock 1) is <= threshold 2");
        assert!((summary.stock_value_at_price - 70.0).abs() < 1e-6);
    }

    #[test]
    fn ai_metadata_upsert_roundtrip() {
        let conn = db();
        seed_item(&conn, "a", "ჩაი", 10.0, 5);
        let meta = AiItemMetadata {
            item_id: "a".into(),
            image_caption_ka: Some("მწვანე ჩაის კოლოფი".into()),
            tags: vec!["tea".into(), "green".into()],
            aliases: vec!["ჩაი".into(), "chai".into()],
            ..Default::default()
        };
        upsert_ai_metadata(&conn, &meta).unwrap();
        let got = get_ai_metadata(&conn, "a").unwrap().unwrap();
        assert_eq!(got.tags, vec!["tea", "green"]);
        assert_eq!(got.image_caption_ka.as_deref(), Some("მწვანე ჩაის კოლოფი"));
    }

    #[test]
    fn category_insert_and_list() {
        let conn = db();
        insert_category(&conn, "c1", &CreateCategoryPayload { name: "სასმელები".into(), color: None, icon: None }).unwrap();
        let cats = get_categories(&conn).unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "სასმელები");
    }

    // ── Extended analytics tests ─────────────────────────────────

    fn append(conn: &Connection, agg: &str, ev: &str, data: &str, hlc: &str) {
        // Mirror production: bump version per (aggregate_id, node_id) pair to
        // satisfy UNIQUE(aggregate_id, node_id, version). Also pin created_at
        // to the HLC value so time-series bucketing uses the test's logical
        // timestamp rather than the wall-clock default.
        let next = get_next_version(conn, agg, "n").unwrap();
        conn.execute(
            "INSERT INTO events (aggregate_id, aggregate_type, event_type, data, hlc_timestamp, node_id, version, created_at)
             VALUES (?1, 'item', ?2, ?3, ?4, 'n', ?5, ?4)",
            params![agg, ev, data, hlc, next],
        )
        .unwrap();
    }

    #[test]
    fn top_sellers_orders_by_sold_count() {
        let conn = db();
        // Two items: a sells more, b sells less but with higher revenue.
        seed_item(&conn, "a", "A", 10.0, 5);
        seed_item(&conn, "b", "B", 50.0, 5);
        // a: 3 sales × 10 = 30 revenue, 3 sold. b: 1 sale × 50 = 50 revenue, 1 sold.
        append(&conn, "a", "SaleRecorded", "{\"quantity\":3,\"sale_price\":10}", "1:0:n");
        append(&conn, "b", "SaleRecorded", "{\"quantity\":1,\"sale_price\":50}", "1:1:n");

        let sellers = get_top_sellers(&conn, 10).unwrap();
        assert_eq!(sellers.len(), 2);
        assert_eq!(sellers[0].id, "a", "higher sold_count wins over higher revenue");
        assert_eq!(sellers[0].sold_count, 3);
    }

    #[test]
    fn dead_stock_excludes_zero_stock_and_sold_items() {
        let conn = db();
        seed_item(&conn, "stuck", "Stuck", 20.0, 10); // dead: stock>0, sold=0
        seed_item(&conn, "sold", "SoldOut", 20.0, 0); // not dead: stock=0
        append(&conn, "sold", "SaleRecorded", "{\"quantity\":10,\"sale_price\":20}", "1:0:n");

        let dead = get_dead_stock(&conn, 10).unwrap();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].id, "stuck");
        assert!((dead[0].locked_value - 200.0).abs() < 1e-6);
    }

    #[test]
    fn category_breakdown_sums_match() {
        let conn = db();
        seed_item(&conn, "a", "A", 10.0, 5);
        seed_item(&conn, "b", "B", 20.0, 3);
        seed_item(&conn, "c", "C", 30.0, 2);
        append(&conn, "a", "SaleRecorded", "{\"quantity\":2,\"sale_price\":10}", "1:0:n");
        append(&conn, "b", "SaleRecorded", "{\"quantity\":1,\"sale_price\":20}", "1:1:n");

        let cats = get_category_breakdown(&conn).unwrap();
        assert!(!cats.is_empty());
        let total_rev: f64 = cats.iter().map(|c| c.revenue).sum();
        assert!((total_rev - 40.0).abs() < 1e-6, "sum of category revenue should equal sum of recorded sales");
    }

    #[test]
    fn sales_timeseries_buckets_by_day() {
        let conn = db();
        seed_item(&conn, "a", "A", 10.0, 5);
        // Two SaleRecorded events on the same day — should collapse into one bucket.
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":2,\"sale_price\":10}",
            "2026-06-15T10:00:00.000",
        );
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":3,\"sale_price\":10}",
            "2026-06-15T18:30:00.000",
        );
        // A different day.
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":1,\"sale_price\":10}",
            "2026-06-16T09:00:00.000",
        );

        let ts = get_sales_timeseries(&conn, "day", None).unwrap();
        assert_eq!(ts.len(), 2);
        assert_eq!(ts[0].bucket, "2026-06-15");
        assert_eq!(ts[0].units_sold, 5);
        assert_eq!(ts[0].sales_count, 2);
        assert_eq!(ts[1].bucket, "2026-06-16");
        assert_eq!(ts[1].units_sold, 1);
    }

    #[test]
    fn sales_timeseries_rejects_invalid_bucket() {
        let conn = db();
        let res = get_sales_timeseries(&conn, "hour", None);
        assert!(res.is_err());
    }

    #[test]
    fn ai_coverage_counts_only_items_with_metadata() {
        let conn = db();
        seed_item(&conn, "a", "A", 10.0, 5);
        seed_item(&conn, "b", "B", 10.0, 5);
        // a: with caption + tags + aliases; b: none.
        let meta = crate::events::types::AiItemMetadata {
            item_id: "a".into(),
            image_caption_ka: Some("სურათი".into()),
            tags: vec!["t".into()],
            aliases: vec!["x".into()],
            ..Default::default()
        };
        upsert_ai_metadata(&conn, &meta).unwrap();

        let cov = get_ai_coverage(&conn).unwrap();
        assert_eq!(cov.total_items, 2);
        assert_eq!(cov.with_caption_ka, 1);
        assert_eq!(cov.with_tags, 1);
        assert_eq!(cov.with_aliases, 1);
        assert!(cov.latest_ai_update.is_some());
    }

    #[test]
    fn recent_activity_filters_event_types() {
        let conn = db();
        seed_item(&conn, "a", "A", 10.0, 5);
        // Internal-looking event type that should NOT appear in the activity feed.
        append(&conn, "a", "ItemUpdated", "{\"name\":\"A2\"}", "1:0:n");
        append(&conn, "a", "SaleRecorded", "{\"quantity\":1,\"sale_price\":10}", "1:1:n");

        let feed = get_recent_activity(&conn, 10).unwrap();
        let types: Vec<&str> = feed.iter().map(|f| f.event_type.as_str()).collect();
        assert!(types.contains(&"SaleRecorded"));
        assert!(!types.contains(&"ItemUpdated"), "internal ItemUpdated must be filtered out");
        assert!(feed[0].summary.contains("sold"), "summary should be human-readable");
    }

    // ── Stock-out forecast tests ────────────────────────────────

    #[test]
    fn stock_out_forecast_excludes_no_sales_and_zero_stock() {
        let conn = db();
        seed_item(&conn, "dead", "Dead", 10.0, 5); // never sold — excluded
        seed_item(&conn, "empty", "Empty", 10.0, 0); // zero stock — excluded
        append(&conn, "empty", "SaleRecorded", "{\"quantity\":5,\"sale_price\":10}", "2026-06-15T10:00:00.000");
        seed_item(&conn, "live", "Live", 10.0, 5);
        append(&conn, "live", "SaleRecorded", "{\"quantity\":1,\"sale_price\":10}", "2026-06-15T10:00:00.000");
        append(&conn, "live", "SaleRecorded", "{\"quantity\":1,\"sale_price\":10}", "2026-06-25T10:00:00.000");

        let fc = get_stock_out_forecast(&conn, 10, 2).unwrap();
        assert_eq!(fc.len(), 1);
        assert_eq!(fc[0].id, "live");
        assert!(fc[0].velocity_per_day > 0.0);
        assert!(fc[0].days_until_stockout.is_finite());
    }

    #[test]
    fn stock_out_forecast_orders_by_urgency() {
        let conn = db();
        // 'fast' sells a lot over a wide window — velocity high → small stockout window
        // 'slow' sells little over the same window → big stockout window
        // Both must keep current_stock > 0 after sales (initial_stock ≥ total sales).
        seed_item(&conn, "fast", "Fast", 10.0, 20);
        seed_item(&conn, "slow", "Slow", 10.0, 20);
        // fast: 10 sales; slow: 5 sales — same time window, different velocities.
        for i in 0..10 {
            append(
                &conn,
                "fast",
                "SaleRecorded",
                "{\"quantity\":1,\"sale_price\":10}",
                &format!("2026-06-{:02}T10:00:00.000", i + 1),
            );
            if i < 5 {
                append(
                    &conn,
                    "slow",
                    "SaleRecorded",
                    "{\"quantity\":1,\"sale_price\":10}",
                    &format!("2026-06-{:02}T10:00:00.000", i + 1),
                );
            }
        }

        let fc = get_stock_out_forecast(&conn, 10, 5).unwrap();
        assert_eq!(fc.len(), 2);
        assert_eq!(fc[0].id, "fast", "fast stockout must come first");
        assert_eq!(fc[1].id, "slow");
        assert!(
            fc[0].days_until_stockout < fc[1].days_until_stockout,
            "fast must run out sooner than slow (fast={}, slow={})",
            fc[0].days_until_stockout,
            fc[1].days_until_stockout
        );
    }

    #[test]
    fn stock_out_forecast_respects_min_sales() {
        let conn = db();
        seed_item(&conn, "single", "Single", 10.0, 5);
        append(
            &conn,
            "single",
            "SaleRecorded",
            "{\"quantity\":1,\"sale_price\":10}",
            "2026-06-15T10:00:00.000",
        );
        // min_sales=2 excludes the single-sale item.
        let fc = get_stock_out_forecast(&conn, 10, 2).unwrap();
        assert!(fc.is_empty());
        // min_sales=1 includes it.
        let fc = get_stock_out_forecast(&conn, 10, 1).unwrap();
        assert_eq!(fc.len(), 1);
    }

    // ── Heatmap tests ────────────────────────────────────────────

    #[test]
    fn heatmap_groups_by_weekday_and_hour() {
        let conn = db();
        seed_item(&conn, "a", "A", 10.0, 50);
        // 2026-06-15 was a Monday (weekday=1). 14:00 hour.
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":3,\"sale_price\":10}",
            "2026-06-15T14:30:00.000",
        );
        // Same hour, different day → another cell.
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":2,\"sale_price\":10}",
            "2026-06-17T14:45:00.000", // Wednesday=3
        );
        // Same weekday, different hour.
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":1,\"sale_price\":10}",
            "2026-06-15T09:00:00.000",
        );

        let hm = get_sales_heatmap(&conn, None).unwrap();
        assert_eq!(hm.len(), 3, "three distinct (weekday,hour) cells");

        // Mon@14:00 should have revenue 30 (3 × 10), units 3.
        let mon14 = hm.iter().find(|c| c.weekday == 1 && c.hour == 14).unwrap();
        assert!((mon14.revenue - 30.0).abs() < 1e-6);
        assert_eq!(mon14.units, 3);

        // Mon@09:00 should be its own cell with revenue 10.
        let mon9 = hm.iter().find(|c| c.weekday == 1 && c.hour == 9).unwrap();
        assert!((mon9.revenue - 10.0).abs() < 1e-6);
    }

    #[test]
    fn heatmap_respects_since_filter() {
        let conn = db();
        seed_item(&conn, "a", "A", 10.0, 20);
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":1,\"sale_price\":10}",
            "2026-06-15T14:00:00.000",
        );
        append(
            &conn,
            "a",
            "SaleRecorded",
            "{\"quantity\":1,\"sale_price\":10}",
            "2026-05-01T14:00:00.000", // before the filter
        );

        let since = Some("2026-06-01");
        let hm = get_sales_heatmap(&conn, since).unwrap();
        assert_eq!(hm.len(), 1);
        assert!((hm[0].revenue - 10.0).abs() < 1e-6);
    }
}
