// AI commands — мост между UI «живого пространства» и AI Gateway.
//
// assistant_query — ядро главного экрана: локальный поиск (FTS) + AI-ответ.
// analyze_item_image — извлечение структурных метаданных из фото товара.
//
// Безопасность: в провайдер уходит только минимальный релевантный контекст
// (топ результатов поиска), не вся БД. Ключи берутся из secrets, не из кода.

use crate::ai::types::{
    AiAnswer, AiProviderKind, FileContext, ImageInput, ProductContext, ProviderStatus,
};
use crate::ai::{self};
use crate::db::Database;
use crate::events::types::AiItemMetadata;
use crate::files::local_context::{read_registered_file, ContextFileAccess};
use crate::search::{self, ProductSearchResponse};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

const MAX_CONTEXT_ITEMS: usize = 8;
const MAX_AI_CONTEXT_FILES: usize = 10;
const MAX_AI_CONTEXT_CHARS: usize = 96 * 1024;

fn selected_provider(conn: &Connection) -> AiProviderKind {
    let s: Option<String> = conn
        .query_row(
            "SELECT value FROM local_config WHERE key = 'ai_provider'",
            [],
            |r| r.get(0),
        )
        .ok();
    AiProviderKind::from_str(s.as_deref().unwrap_or("mock"))
}

#[derive(Debug, Serialize)]
pub struct AssistantResponse {
    pub mode: String,
    pub language: String,
    pub search: ProductSearchResponse,
    pub answer: Option<AiAnswer>,
    /// Controlled error string if the AI layer failed (UI still shows cards).
    pub answer_error: Option<String>,
}

/// Main "void interface" command: local product search + grounded AI answer.
#[tauri::command]
pub async fn assistant_query(
    db: State<'_, Database>,
    file_access: State<'_, ContextFileAccess>,
    query: String,
    language: Option<String>,
    context_file_ids: Option<Vec<String>>,
) -> Result<AssistantResponse, String> {
    // Pre-await: run search + read selected provider, then drop the lock.
    let (search_resp, selected) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let resp = search::search_products(&conn, &query, Some(40))?;
        (resp, selected_provider(&conn))
    };

    let lang = language.unwrap_or_else(|| search_resp.language.clone());

    let requested_ids = context_file_ids.unwrap_or_default();
    if requested_ids.len() > MAX_AI_CONTEXT_FILES {
        return Err(format!(
            "Select at most {MAX_AI_CONTEXT_FILES} files for one AI request."
        ));
    }
    let mut unique_ids = HashSet::new();
    let registered_files = requested_ids
        .iter()
        .filter(|id| unique_ids.insert((*id).clone()))
        .map(|id| file_access.resolve(id).map_err(|error| error.message))
        .collect::<Result<Vec<_>, _>>()?;

    // The picker accepts reasonably sized source files for local workflows,
    // while provider context is capped separately to prevent huge requests.
    let mut remaining_chars = MAX_AI_CONTEXT_CHARS;
    let mut context_files = Vec::with_capacity(registered_files.len());
    let registered_count = registered_files.len();
    for (index, registered) in registered_files.into_iter().enumerate() {
        let document = read_registered_file(registered)
            .await
            .map_err(|error| error.message)?;
        let total_chars = document.content.chars().count();
        // Share the remaining budget so one large first file cannot crowd out
        // every later attachment; unused space rolls forward.
        let remaining_files = registered_count - index;
        let file_budget = remaining_chars / remaining_files;
        let take_chars = total_chars.min(file_budget);
        let content = document.content.chars().take(take_chars).collect();
        context_files.push(FileContext {
            name: document.file.file_name,
            content,
            truncated: take_chars < total_chars,
        });
        remaining_chars -= take_chars;
    }

    // Minimal grounding context — top N hits only (never the whole DB).
    let context_items: Vec<ProductContext> = search_resp
        .results
        .iter()
        .take(MAX_CONTEXT_ITEMS)
        .map(|r| ProductContext {
            item_id: r.item.id.clone(),
            name: r.item.name.clone(),
            description: r.item.description.clone(),
            category: r.item.category.clone(),
            price: r.item.current_price,
            tags: r.matched_by.clone(),
        })
        .collect();

    let router = ai::build_router(selected);
    let req = crate::ai::types::AiRequest {
        query,
        language: lang.clone(),
        context_items,
        context_files,
    };

    let (answer, answer_error) = match router.answer(&req).await {
        Ok(a) => (Some(a), None),
        Err(e) => (None, Some(e.to_string())),
    };

    Ok(AssistantResponse {
        mode: "assistant".to_string(),
        language: lang,
        search: search_resp,
        answer,
        answer_error,
    })
}

/// Sniff image MIME from magic bytes (we store images without an extension).
fn sniff_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else {
        "image/jpeg"
    }
}

/// Analyze the stored photo of an item, persist AI metadata, reindex FTS.
#[tauri::command]
pub async fn analyze_item_image(
    handle: AppHandle,
    db: State<'_, Database>,
    item_id: String,
) -> Result<AiItemMetadata, String> {
    // Pre-await: resolve image path + selected provider; clone Arc for post-await writes.
    let conn_arc: Arc<_> = Arc::clone(&db.conn);
    let (image_rel, item_name, selected) = {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        let item = crate::db::queries::get_item_by_id(&conn, &item_id)?
            .ok_or_else(|| "item not found".to_string())?;
        let rel = item
            .image_path
            .ok_or_else(|| "item has no image".to_string())?;
        (rel, item.name, selected_provider(&conn))
    };

    let app_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let abs = app_dir.join(&image_rel);
    let bytes = tokio::fs::read(&abs)
        .await
        .map_err(|e| format!("read image: {e}"))?;
    let mime = sniff_mime(&bytes).to_string();

    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let b64 = STANDARD.encode(&bytes);
    let image = ImageInput { mime, base64: b64 };

    let router = ai::build_router(selected);
    let meta = router
        .analyze_image(&image, Some(&item_name))
        .await
        .map_err(|e| e.to_string())?;

    // Best-effort Georgian second-pass on the KA caption. Captures spec
    // requirement #13 automatically without UI churn. If the provider fails
    // (or no provider is configured), georgian_review returns the input as-is,
    // so this never blocks the analyze flow.
    let caption_ka = match (router.first(), meta.caption_ka.as_deref()) {
        (Some(p), Some(text)) => ai::localization::georgian_review(p, text).await,
        _ => meta.caption_ka.clone().unwrap_or_default(),
    };

    let ai_meta = AiItemMetadata {
        item_id: item_id.clone(),
        image_caption_ru: meta.caption_ru,
        image_caption_ka: if caption_ka.is_empty() {
            None
        } else {
            Some(caption_ka)
        },
        image_caption_en: meta.caption_en,
        tags: meta.tags,
        visual_attributes: meta.visual_attributes,
        aliases: meta.aliases,
        ..Default::default()
    };

    // Post-await: persist + reindex (Arc clone, no State held across await).
    {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        crate::db::queries::upsert_ai_metadata(&conn, &ai_meta)?;
        let _ = search::reindex_item_fts(&conn, &item_id);
    }
    Ok(ai_meta)
}

#[derive(Debug, Serialize)]
pub struct AiStatus {
    pub selected: String,
    pub providers: Vec<ProviderStatus>,
}

#[tauri::command]
pub fn ai_get_status(db: State<'_, Database>) -> Result<AiStatus, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let selected = selected_provider(&conn);
    Ok(AiStatus {
        selected: selected.as_str().to_string(),
        providers: ai::provider_statuses(selected),
    })
}

#[tauri::command]
pub fn ai_set_provider(db: State<'_, Database>, provider: String) -> Result<(), String> {
    let kind = AiProviderKind::from_str(&provider);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO local_config (key, value) VALUES ('ai_provider', ?1)",
        [kind.as_str()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn ai_set_provider_key(provider: String, key: String) -> Result<(), String> {
    let kind = AiProviderKind::from_str(&provider);
    ai::secrets::store_key(kind, &key)
}

#[tauri::command]
pub fn ai_delete_provider_key(provider: String) -> Result<(), String> {
    let kind = AiProviderKind::from_str(&provider);
    ai::secrets::delete_key(kind)
}
