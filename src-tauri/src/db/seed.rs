// Optional demo seed — небольшой набор грузинских товаров, чтобы «живое
// пространство» было чем наполнить при первом запуске и для smoke-тестов
// поиска. Идемпотентно: ничего не делает, если товары уже есть.

use rusqlite::Connection;

struct DemoItem {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    category: &'static str,
    price: f64,
    stock: i64,
    tags: &'static [&'static str],
    aliases: &'static [&'static str],
    caption_ka: &'static str,
}

const DEMO: &[DemoItem] = &[
    DemoItem {
        id: "demo-jeep",
        name: "სათამაშო ჯიპი",
        description: "წითელი მეტალის სათამაშო ჯიპი, 4x4 მოდელი",
        category: "სათამაშოები",
        price: 45.0,
        stock: 12,
        tags: &["jeep", "toy", "red", "car"],
        aliases: &["ჯიპი", "მანქანა", "jeep", "джип", "машина"],
        caption_ka: "წითელი სათამაშო ჯიპი დიდი ბორბლებით",
    },
    DemoItem {
        id: "demo-basket",
        name: "სასაჩუქრე კალათა",
        description: "სადღესასწაულო სასაჩუქრე კალათა ტკბილეულით",
        category: "საჩუქრები",
        price: 120.0,
        stock: 6,
        tags: &["gift", "basket", "holiday"],
        aliases: &["კალათა", "საჩუქარი", "подарочная корзина", "gift basket"],
        caption_ka: "სასაჩუქრე კალათა წითელი ლენტით",
    },
    DemoItem {
        id: "demo-tea",
        name: "მწვანე ჩაი",
        description: "ბიო მწვანე ჩაი, 100გ კოლოფი",
        category: "სასმელები",
        price: 18.5,
        stock: 30,
        tags: &["tea", "green", "organic"],
        aliases: &["ჩაი", "чай", "tea"],
        caption_ka: "მწვანე ჩაის მწვანე კოლოფი",
    },
    DemoItem {
        id: "demo-redbox",
        name: "წითელი ყუთი",
        description: "მუყაოს წითელი შესაფუთი ყუთი, საშუალო ზომა",
        category: "შეფუთვა",
        price: 7.0,
        stock: 200,
        tags: &["box", "red", "packaging"],
        aliases: &["ყუთი", "კოლოფი", "красная коробка", "red box"],
        caption_ka: "წითელი მუყაოს ყუთი",
    },
];

/// Insert demo items (+ AI metadata + FTS index). No-op if items already exist.
pub fn seed_demo_items(conn: &Connection) -> Result<usize, String> {
    let existing: i64 = conn
        .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if existing > 0 {
        return Ok(0);
    }

    let mut inserted = 0usize;
    for d in DEMO {
        let data = serde_json::json!({
            "name": d.name,
            "description": d.description,
            "category": d.category,
            "price": d.price,
            "initial_stock": d.stock,
        });
        conn.execute(
            "INSERT INTO events (aggregate_id, aggregate_type, event_type, data, hlc_timestamp, node_id, version)
             VALUES (?1, 'item', 'ItemCreated', ?2, ?3, 'seed', 1)",
            rusqlite::params![d.id, data.to_string(), format!("0:{inserted}:seed")],
        )
        .map_err(|e| e.to_string())?;

        let meta = crate::events::types::AiItemMetadata {
            item_id: d.id.to_string(),
            image_caption_ka: Some(d.caption_ka.to_string()),
            tags: d.tags.iter().map(|s| s.to_string()).collect(),
            aliases: d.aliases.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        crate::db::queries::upsert_ai_metadata(conn, &meta)?;
        crate::search::reindex_item_fts(conn, d.id)?;
        inserted += 1;
    }
    log::info!("Seeded {} demo items", inserted);
    Ok(inserted)
}
