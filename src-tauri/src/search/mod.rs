// Product search — FTS5 over name/description/category + AI tags/caption/aliases.
//
// Контракт ответа повторяет JSON из задания (`mode: "product_search"`),
// плюс к каждому результату прикладывается полный `item`, чтобы UI мог
// построить карточку без дополнительного запроса.
//
// Индекс обслуживается из Rust (reindex_item_fts), а не SQL-триггерами —
// так надёжнее склеивать данные из items и ai_item_metadata.

use crate::events::types::Item;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProductSearchResult {
    pub item_id: String,
    pub title: String,
    pub reason: String,
    pub confidence: f64,
    pub matched_by: Vec<String>,
    /// primary | quiet | visual | dense
    pub card_style_hint: String,
    /// Full projection row so the UI can render the card directly.
    pub item: Item,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductSearchResponse {
    pub mode: String,
    pub query: String,
    /// ru | ka | en
    pub language: String,
    pub results: Vec<ProductSearchResult>,
    pub assistant_summary: String,
}

/// Rebuild the FTS row for one item from items + ai_item_metadata.
pub fn reindex_item_fts(conn: &Connection, item_id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM item_search_fts WHERE item_id = ?1", [item_id])
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO item_search_fts (item_id, name, description, category, ai_tags, ai_caption, aliases)
         SELECT i.id, i.name, i.description, i.category,
            COALESCE((SELECT tags_json FROM ai_item_metadata WHERE item_id = i.id), ''),
            COALESCE((SELECT
                COALESCE(image_caption_ka,'') || ' ' || COALESCE(image_caption_ru,'') || ' ' || COALESCE(image_caption_en,'')
                FROM ai_item_metadata WHERE item_id = i.id), ''),
            COALESCE((SELECT aliases_json FROM ai_item_metadata WHERE item_id = i.id), '')
         FROM items i WHERE i.id = ?1",
        [item_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Rebuild the entire FTS index. Returns the number of indexed items.
pub fn rebuild_search_index(conn: &Connection) -> Result<usize, String> {
    conn.execute("DELETE FROM item_search_fts", [])
        .map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id FROM items").map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    for id in &ids {
        reindex_item_fts(conn, id)?;
    }
    Ok(ids.len())
}

/// Detect the dominant script of the query to drive localization. Heuristic:
/// Georgian (Mkhedruli) → "ka", Cyrillic → "ru", otherwise → "en".
pub fn detect_language(text: &str) -> &'static str {
    let mut ka = 0usize;
    let mut ru = 0usize;
    let mut latin = 0usize;
    for c in text.chars() {
        let u = c as u32;
        if (0x10A0..=0x10FF).contains(&u) {
            ka += 1;
        } else if (0x0400..=0x04FF).contains(&u) {
            ru += 1;
        } else if c.is_ascii_alphabetic() {
            latin += 1;
        }
    }
    if ka >= ru && ka >= latin && ka > 0 {
        "ka"
    } else if ru >= latin && ru > 0 {
        "ru"
    } else {
        "en"
    }
}

/// Turn a free-text query into a safe FTS5 MATCH expression: keep only
/// letters/digits (any script), make each term a prefix query, OR them
/// together for recall. Returns None if nothing usable remains.
fn build_fts_match(query: &str) -> Option<String> {
    let terms: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t.to_lowercase()))
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" OR "))
    }
}

/// Compute which source fields contain any of the query terms (best-effort,
/// done in Rust after hydration so we can report `matched_by` precisely).
fn matched_fields(item: &Item, meta: Option<&crate::events::types::AiItemMetadata>, terms: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let name = item.name.to_lowercase();
    let desc = item.description.to_lowercase();
    let cat = item.category.to_lowercase();

    let hit = |hay: &str| terms.iter().any(|t| hay.contains(t.as_str()));

    if hit(&name) {
        out.push("name".to_string());
    }
    if hit(&desc) {
        out.push("description".to_string());
    }
    if hit(&cat) {
        out.push("category".to_string());
    }
    if let Some(m) = meta {
        let tags = m.tags.join(" ").to_lowercase();
        let aliases = m.aliases.join(" ").to_lowercase();
        let caption = [
            m.image_caption_ka.clone().unwrap_or_default(),
            m.image_caption_ru.clone().unwrap_or_default(),
            m.image_caption_en.clone().unwrap_or_default(),
        ]
        .join(" ")
        .to_lowercase();
        if hit(&tags) {
            out.push("ai_tags".to_string());
        }
        if hit(&aliases) {
            out.push("aliases".to_string());
        }
        if hit(&caption) {
            out.push("image".to_string());
        }
    }
    if out.is_empty() {
        out.push("name".to_string());
    }
    out
}

fn card_hint(item: &Item, confidence: f64, total: usize) -> String {
    if item.image_path.is_some() {
        "visual".to_string()
    } else if confidence >= 0.6 {
        "primary".to_string()
    } else if total >= 8 {
        "dense".to_string()
    } else {
        "quiet".to_string()
    }
}

fn assistant_summary(lang: &str, query: &str, n: usize) -> String {
    match lang {
        "ka" => {
            if n == 0 {
                format!("ვერაფერი მოიძებნა მოთხოვნაზე «{}».", query)
            } else {
                format!("ნაპოვნია {} შედეგი მოთხოვნაზე «{}».", n, query)
            }
        }
        "ru" => {
            if n == 0 {
                format!("По запросу «{}» ничего не найдено.", query)
            } else {
                format!("Найдено результатов: {} по запросу «{}».", n, query)
            }
        }
        _ => {
            if n == 0 {
                format!("No matches for “{}”.", query)
            } else {
                format!("Found {} result(s) for “{}”.", n, query)
            }
        }
    }
}

fn reason_for(lang: &str, matched: &[String]) -> String {
    // Returns a 'static label so closure lifetimes don't tangle with `matched`.
    let label = |f: &str| -> &'static str {
        match (lang, f) {
            ("ka", "name") => "სახელი",
            ("ka", "description") => "აღწერა",
            ("ka", "category") => "კატეგორია",
            ("ka", "ai_tags") => "ტეგები",
            ("ka", "aliases") => "სინონიმები",
            ("ka", "image") => "ფოტო",
            ("ru", "name") => "название",
            ("ru", "description") => "описание",
            ("ru", "category") => "категория",
            ("ru", "ai_tags") => "теги",
            ("ru", "aliases") => "синонимы",
            ("ru", "image") => "фото",
            (_, "name") => "name",
            (_, "description") => "description",
            (_, "category") => "category",
            (_, "ai_tags") => "tags",
            (_, "aliases") => "aliases",
            (_, "image") => "image",
            _ => "?",
        }
    };
    let parts: Vec<&str> = matched.iter().map(|m| label(m)).collect();
    let prefix = match lang {
        "ka" => "ემთხვევა: ",
        "ru" => "совпадение: ",
        _ => "matched: ",
    };
    format!("{}{}", prefix, parts.join(", "))
}

/// Run a product search. Tries FTS5 first; falls back to LIKE when the query
/// produces no FTS terms or FTS errors out (e.g. exotic input).
pub fn search_products(
    conn: &Connection,
    query: &str,
    limit: Option<u32>,
) -> Result<ProductSearchResponse, String> {
    let lang = detect_language(query);
    let cap = limit.unwrap_or(40).min(200);
    let terms: Vec<String> = query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect();

    // (item_id, bm25 score). Lower score = better match.
    let mut scored: Vec<(String, f64)> = Vec::new();

    if let Some(match_expr) = build_fts_match(query) {
        let fts_result = (|| -> rusqlite::Result<Vec<(String, f64)>> {
            let mut stmt = conn.prepare(
                "SELECT item_id, bm25(item_search_fts) AS score
                 FROM item_search_fts
                 WHERE item_search_fts MATCH ?1
                 ORDER BY score
                 LIMIT ?2",
            )?;
            let rows = stmt
                .query_map(rusqlite::params![match_expr, cap as i64], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })();

        match fts_result {
            Ok(rows) => scored = rows,
            Err(e) => log::warn!("FTS search failed ({}); falling back to LIKE", e),
        }
    }

    // Fallback: substring match against name/description/category.
    if scored.is_empty() && !terms.is_empty() {
        let like = format!("%{}%", terms.join("%"));
        let mut stmt = conn
            .prepare(
                "SELECT id FROM items
                 WHERE lower(name) LIKE ?1 OR lower(description) LIKE ?1 OR lower(category) LIKE ?1
                 ORDER BY updated_at DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map(rusqlite::params![like, cap as i64], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        scored = ids.into_iter().map(|id| (id, -0.2)).collect();
    }

    let total = scored.len();
    let mut results = Vec::with_capacity(total);
    for (item_id, score) in scored {
        let Some(item) = crate::db::queries::get_item_by_id(conn, &item_id)? else {
            continue;
        };
        let meta = crate::db::queries::get_ai_metadata(conn, &item_id)?;
        // Map bm25 magnitude into a 0..1 confidence (monotonic, bounded).
        let mag = score.abs();
        let confidence = (mag / (mag + 1.0)).clamp(0.05, 0.99);
        let matched = matched_fields(&item, meta.as_ref(), &terms);
        let hint = card_hint(&item, confidence, total);
        results.push(ProductSearchResult {
            item_id: item.id.clone(),
            title: item.name.clone(),
            reason: reason_for(lang, &matched),
            confidence,
            matched_by: matched,
            card_style_hint: hint,
            item,
        });
    }

    Ok(ProductSearchResponse {
        mode: "product_search".to_string(),
        query: query.to_string(),
        language: lang.to_string(),
        assistant_summary: assistant_summary(lang, query, results.len()),
        results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    fn add(conn: &Connection, id: &str, name: &str, desc: &str, cat: &str, aliases: &[&str], tags: &[&str]) {
        conn.execute(
            "INSERT INTO events (aggregate_id, aggregate_type, event_type, data, hlc_timestamp, node_id, version)
             VALUES (?1, 'item', 'ItemCreated', ?2, '0:0:n', 'n', 1)",
            params![id, serde_json::json!({"name": name, "description": desc, "category": cat, "price": 1.0}).to_string()],
        )
        .unwrap();
        let meta = crate::events::types::AiItemMetadata {
            item_id: id.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        crate::db::queries::upsert_ai_metadata(conn, &meta).unwrap();
        reindex_item_fts(conn, id).unwrap();
    }

    #[test]
    fn detect_language_by_script() {
        assert_eq!(detect_language("მანქანა"), "ka");
        assert_eq!(detect_language("красная коробка"), "ru");
        assert_eq!(detect_language("gift basket"), "en");
    }

    #[test]
    fn search_matches_alias_across_languages() {
        let conn = db();
        add(&conn, "jeep", "სათამაშო ჯიპი", "red toy jeep", "სათამაშოები", &["ჯიპი", "машина", "jeep"], &["car", "red"]);
        add(&conn, "tea", "მწვანე ჩაი", "green tea box", "სასმელები", &["ჩაი", "чай"], &["tea"]);

        // Russian alias should find the Georgian-named jeep.
        let resp = search_products(&conn, "машина", None).unwrap();
        assert_eq!(resp.language, "ru");
        assert!(resp.results.iter().any(|r| r.item_id == "jeep"), "alias search must hit jeep");

        // English term.
        let resp2 = search_products(&conn, "jeep", None).unwrap();
        assert!(resp2.results.iter().any(|r| r.item_id == "jeep"));
        assert!(resp2.results[0].matched_by.contains(&"aliases".to_string())
            || resp2.results[0].matched_by.contains(&"name".to_string()));
    }

    #[test]
    fn empty_query_returns_no_results_gracefully() {
        let conn = db();
        add(&conn, "tea", "ჩაი", "tea", "x", &[], &[]);
        let resp = search_products(&conn, "!!!", None).unwrap();
        assert_eq!(resp.results.len(), 0);
    }
}
