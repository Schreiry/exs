// Mock provider — детерминированные ответы без сети. Используется когда нет ключей
// (dev) и в тестах. Always configured.

use crate::ai::provider::AiProvider;
use crate::ai::types::{AiAnswer, AiError, AiProviderKind, AiRequest, ImageInput, VisionMetadata};
use async_trait::async_trait;

pub struct MockProvider;

#[async_trait]
impl AiProvider for MockProvider {
    fn kind(&self) -> AiProviderKind {
        AiProviderKind::Mock
    }

    fn is_configured(&self) -> bool {
        true
    }

    async fn answer(&self, req: &AiRequest) -> Result<AiAnswer, AiError> {
        let n = req.context_items.len();
        let text = match req.language.as_str() {
            "ka" => format!("(mock) მოთხოვნა: «{}». ნაპოვნია {n} შესაბამისი პროდუქტი.", req.query),
            "ru" => format!("(mock) Запрос: «{}». Найдено {n} подходящих товаров.", req.query),
            _ => format!("(mock) Query: \"{}\". {n} relevant products.", req.query),
        };
        Ok(AiAnswer {
            text,
            language: req.language.clone(),
            provider: "mock".to_string(),
        })
    }

    async fn analyze_image(
        &self,
        _image: &ImageInput,
        hint: Option<&str>,
    ) -> Result<VisionMetadata, AiError> {
        if let Some(h) = hint {
            log::debug!("mock vision hint: {h}");
        }
        Ok(VisionMetadata {
            caption_ka: Some("მოკ პროდუქტი".to_string()),
            caption_ru: Some("мок-товар".to_string()),
            caption_en: Some("mock product".to_string()),
            tags: vec!["mock".to_string(), "demo".to_string()],
            aliases: vec!["პროდუქტი".to_string(), "товар".to_string(), "product".to_string()],
            visual_attributes: serde_json::json!({"color": "grey", "material": "", "shape": "", "size": ""}),
            confidence: 0.42,
            ..Default::default()
        })
    }
}
