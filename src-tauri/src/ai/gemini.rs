// Gemini provider (Google Generative Language API). Fallback / альтернатива для
// vision и structured output. Ключ передаётся как query-параметр ?key=.
// Model: gemini-flash-latest (auto-updating alias). Override per install via the
// GEMINI_MODEL env var. Vision calls request application/json via
// generationConfig.responseMimeType — consistent with OpenAI's response_format.

use crate::ai::provider::AiProvider;
use crate::ai::types::{AiAnswer, AiError, AiProviderKind, AiRequest, ImageInput, VisionMetadata};
use crate::ai::{prompts, vision};
use async_trait::async_trait;
use serde_json::json;

const BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const DEFAULT_MODEL: &str = "gemini-flash-latest";
const MODEL_ENV: &str = "GEMINI_MODEL";

pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model
                .or_else(|| {
                    std::env::var(MODEL_ENV)
                        .ok()
                        .filter(|s| !s.trim().is_empty())
                })
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            client: super::http_client(),
        }
    }

    async fn generate(&self, body: serde_json::Value) -> Result<String, AiError> {
        let url = format!("{BASE}/{}:generateContent?key={}", self.model, self.api_key);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;
        if !status.is_success() {
            log::warn!("Gemini returned HTTP {}", status);
            return Err(AiError::Provider(format!("HTTP {status}")));
        }
        let v: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;
        v["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                AiError::Parse("missing candidates[0].content.parts[0].text".to_string())
            })
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn kind(&self) -> AiProviderKind {
        AiProviderKind::Gemini
    }

    fn is_configured(&self) -> bool {
        !self.api_key.trim().is_empty()
    }

    async fn answer(&self, req: &AiRequest) -> Result<AiAnswer, AiError> {
        let prompt = format!(
            "{}\n\n{}",
            prompts::system_answer(&req.language),
            prompts::answer_user_message(req)
        );
        let body = json!({ "contents": [{ "parts": [{ "text": prompt }] }] });
        let text = self.generate(body).await?;
        Ok(AiAnswer {
            text,
            language: req.language.clone(),
            provider: "gemini".to_string(),
        })
    }

    async fn analyze_image(
        &self,
        image: &ImageInput,
        hint: Option<&str>,
    ) -> Result<VisionMetadata, AiError> {
        let body = json!({
            "contents": [{
                "parts": [
                    { "text": prompts::vision_instruction(hint) },
                    { "inline_data": { "mime_type": image.mime, "data": image.base64 } }
                ]
            }],
            "generationConfig": { "responseMimeType": "application/json" }
        });
        let text = self.generate(body).await?;
        vision::parse_vision_json(&text)
    }
}
