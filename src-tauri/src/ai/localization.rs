// Грузинская локализация: второй проход самопроверки для пользовательского текста.
// Best-effort — при ошибке возвращаем исходный текст (не блокируем UX).

use crate::ai::prompts;
use crate::ai::provider::AiProvider;
use crate::ai::types::AiRequest;

/// Run a Georgian self-review pass over `text` using the given provider.
/// Returns the improved text, or the original if the provider fails.
/// Public API for the `ai` branch (KA second-pass); not yet wired into commands.
#[allow(dead_code)]
pub async fn georgian_review(provider: &dyn AiProvider, text: &str) -> String {
    if text.trim().is_empty() {
        return text.to_string();
    }
    let req = AiRequest {
        query: prompts::georgian_review_instruction(text),
        language: "ka".to_string(),
        context_items: Vec::new(),
    };
    match provider.answer(&req).await {
        Ok(ans) if !ans.text.trim().is_empty() => ans.text,
        _ => text.to_string(),
    }
}
