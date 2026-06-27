// Optional demo seed — небольшой набор грузинских товаров, чтобы «живое
// пространство» было чем наполнить при первом запуске и для smoke-тестов
// поиска. Идемпотентно: ничего не делает, если товары уже есть.
//
// Раньше здесь был «მწვანე ჩაი»; теперь — «ბიპლანი» (деревянный
// механический авиамодель) с большим набором ключей-синонимов на грузинском,
// русском и английском, чтобы поиск по свободному запросу («самолёт»,
// «თვითმფრენი», «biplane», «деревянный конструктор») находил карточку
// даже когда название не совпадает буквально.

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
        // ხის მექანიკური 3D-კონსტრუქტორი — ბიპლანი.
        // Алиасы намеренно широкие: по любому из них свободный поиск
        // должен поднимать эту карточку (см. search::reindex_item_fts).
        id: "demo-biplane",
        name: "ბიპლანი",
        description:
            "ხის მექანიკური 3D-კონსტრუქტორი — ორფრთიანი თვითმფრენი (ბიპლანი) პროპელერით, ასაწყობი მოდელი",
        category: "მოდელები",
        price: 89.0,
        stock: 14,
        tags: &[
            // EN
            "biplane", "airplane", "aeroplane", "aircraft", "propeller", "propeller-plane",
            "vintage-plane", "retro-plane", "wooden-model", "wooden-kit", "mechanical-model",
            "3d-puzzle", "diy", "collectible", "toy", "gift", "souvenir", "aviation",
            // RU
            "самолет", "биплан", "аэроплан", "самолетик", "авиамодель", "пропеллер",
            "деревянная-модель", "деревянный-конструктор", "механическая-модель",
            "винтаж", "ретро", "коллекционная-модель", "сувенир", "игрушка", "подарок",
            // KA (латиницей)
            "biplani", "tskhitmpreni", "sahaero-khmalди",
        ],
        aliases: &[
            // ===== Грузинский: авиация в целом =====
            "ბიპლანი", "თვითმფრენი", "საჰაერო ხომალდი", "ავიახომალდი",
            "აეროპლანი", "საფრენი აპარატი", "ფრთიანი მანქანა",
            "საჰაერო მანქანა", "საჰაერო ტრანსპორტი",
            "ფრენა", "ფრენის აპარატი", "ავიაცია", "ავიამოდელი",
            "მფრენი",
            // ===== Грузинский: части самолёта =====
            "პროპელერი", "პროპელერიანი", "ძრავი", "ფრთები", "ფრთიანი",
            "ორფრთიანი", "კაბინა", "სალტე", "პილოტი", "შასი",
            // ===== Грузинский: модель/конструктор =====
            "მოდელი", "ხის მოდელი", "მექანიკური მოდელი", "3D მოდელი",
            "ხის კონსტრუქტორი", "კონსტრუქტორი", "ასაწყობი", "ასაკრეფი",
            "სამშენებლო ნაკრები", "ხელსაქმის მოდელი", "მოდელიზმი",
            // ===== Грузинский: стиль/назначение =====
            "ვინტაჟი", "რეტრო", "კლასიკური", "ძველი", "ისტორიული",
            "კოლექციური", "კოლექციონერული", "დეკორატიული",
            "საჩუქარი", "სასაჩუქრე", "სუვენირი", "სათამაშო",
            "საბავშვო", "გასართობი", "ჰობი",
            // ===== Грузинский: материал =====
            "ხის", "ხე", "ფანერა", "მუყაო",
            // ===== Русский =====
            "самолет", "самолёт", "самолетик", "биплан", "аэроплан",
            "летательный аппарат", "воздушное судно", "авиация",
            "авиамодель", "модель самолета", "модель самолёта",
            "винтовой самолёт", "винтовой самолет", "пропеллер",
            "деревянная модель", "деревянный конструктор", "деревянная игрушка",
            "механическая модель", "3д-пазл", "3d-пазл",
            "винтаж", "ретро", "классика", "коллекционная модель",
            "коллекционный", "сувенир", "игрушка", "подарок", "хобби",
            "двухкрылый", "двухкрылый самолёт",
            // ===== Английский =====
            "biplane", "airplane", "aeroplane", "aircraft", "propeller plane",
            "propeller", "double-wing", "twin-wing", "winged", "pilot",
            "wooden model", "wooden kit", "wooden puzzle", "mechanical model",
            "3d puzzle", "3d-puzzle", "diy", "model kit", "assembly kit",
            "vintage plane", "retro plane", "classic plane", "collectible",
            "collectible model", "souvenir", "toy", "gift", "hobby",
            "aviation model", "aircraft model", "scale model",
        ],
        caption_ka:
            "ხის მექანიკური 3D-მოდელი — ორფრთიანი თვითმფრენი (ბიპლანი) პროპელერით, ღია ფერის ფანერისგან",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    #[test]
    fn seed_inserts_biplane_with_many_aliases() {
        let conn = fresh_db();
        let n = seed_demo_items(&conn).unwrap();
        assert_eq!(n, 4);

        let (name, category): (String, String) = conn
            .query_row(
                "SELECT name, category FROM items WHERE id = 'demo-biplane'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "ბიპლანი");
        assert_eq!(category, "მოდელები");

        // Aliases should cover Georgian, Russian and English queries.
        let aliases_json: String = conn
            .query_row(
                "SELECT aliases_json FROM ai_item_metadata WHERE item_id = 'demo-biplane'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let aliases: Vec<String> = serde_json::from_str(&aliases_json).unwrap();
        for must in [
            "ბიპლანი",
            "თვითმფრენი",
            "самолет",
            "биплан",
            "biplane",
            "airplane",
            "ხის კონსტრუქტორი",
            "wooden model",
            "ვინტაჟი",
        ] {
            assert!(
                aliases.iter().any(|a| a == must),
                "missing alias `{must}`; got {} aliases",
                aliases.len()
            );
        }
        assert!(
            aliases.len() >= 50,
            "expected a rich alias list, got {}",
            aliases.len()
        );
    }

    #[test]
    fn fts_finds_biplane_by_georgian_alias() {
        let conn = fresh_db();
        seed_demo_items(&conn).unwrap();

        // FTS index uses prefix-match per term, OR'd.
        // "თვითმფრენი" (airplane) should match the biplane item via aliases.
        let hits: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM item_search_fts WHERE item_search_fts MATCH '\"თვითმფრენი\"*'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(hits >= 1, "თვითმფრენი should hit demo-biplane");

        let hits2: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM item_search_fts WHERE item_search_fts MATCH '\"самолет\"*'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(hits2 >= 1, "самолет should hit demo-biplane");
    }

    #[test]
    fn seed_is_idempotent() {
        let conn = fresh_db();
        assert_eq!(seed_demo_items(&conn).unwrap(), 4);
        assert_eq!(seed_demo_items(&conn).unwrap(), 0);
    }
}
