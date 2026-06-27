// Адаптировано из Exsul (src-tauri/src/db/queries.rs). Оставлено только то,
// что нужно ядру AI-ассистента: товары, категории, audit, события, AI-метаданные
// и бизнес-аналитика. Тяжёлый доменный SQL (orders/flowers/...) не переносился.

use crate::events::types::{
    AiItemMetadata, AuditLog, AuditLogFilter, Category, CategoryCount, CreateCategoryPayload,
    EventRecord, InventorySummary, Item, ListPage,
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
}
