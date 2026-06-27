// AiRouter — пробует провайдеров по порядку (primary → fallbacks). Если все
// недоступны, возвращает controlled error (последнюю ошибку), а не падает.

use crate::ai::provider::AiProvider;
use crate::ai::types::{AiAnswer, AiError, AiRequest, ImageInput, VisionMetadata};

pub struct AiRouter {
    providers: Vec<Box<dyn AiProvider>>,
}

impl AiRouter {
    pub fn new(providers: Vec<Box<dyn AiProvider>>) -> Self {
        Self { providers }
    }

    #[allow(dead_code)]
    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|p| p.kind().as_str().to_string())
            .collect()
    }

    pub async fn answer(&self, req: &AiRequest) -> Result<AiAnswer, AiError> {
        let mut last = AiError::NotConfigured("no AI provider configured".to_string());
        for p in &self.providers {
            match p.answer(req).await {
                Ok(a) => return Ok(a),
                Err(e) => {
                    log::warn!("provider {} failed: {}", p.kind().as_str(), e);
                    last = e;
                }
            }
        }
        Err(last)
    }

    pub async fn analyze_image(
        &self,
        image: &ImageInput,
        hint: Option<&str>,
    ) -> Result<VisionMetadata, AiError> {
        let mut last = AiError::NotConfigured("no AI provider configured".to_string());
        for p in &self.providers {
            match p.analyze_image(image, hint).await {
                Ok(m) => return Ok(m),
                Err(e) => {
                    log::warn!("provider {} vision failed: {}", p.kind().as_str(), e);
                    last = e;
                }
            }
        }
        Err(last)
    }

    /// Borrow the first provider — used by the Georgian self-review pass.
    #[allow(dead_code)]
    pub fn first(&self) -> Option<&dyn AiProvider> {
        self.providers.first().map(|b| b.as_ref())
    }
}
