// AiProvider — провайдер-агностичный интерфейс. Каждый конкретный провайдер
// (openai/gemini/claude/mock) реализует этот trait. Router держит набор таких
// провайдеров и обеспечивает fallback.

use crate::ai::types::{AiAnswer, AiError, AiProviderKind, AiRequest, ImageInput, VisionMetadata};
use async_trait::async_trait;

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn kind(&self) -> AiProviderKind;

    /// True when the provider has the credentials it needs to make a call.
    /// Used by the Settings/status surface (ai branch).
    #[allow(dead_code)]
    fn is_configured(&self) -> bool;

    /// Generate a text answer grounded in the request context.
    async fn answer(&self, req: &AiRequest) -> Result<AiAnswer, AiError>;

    /// Analyze a single product image and return structured metadata.
    async fn analyze_image(
        &self,
        image: &ImageInput,
        hint: Option<&str>,
    ) -> Result<VisionMetadata, AiError>;
}
