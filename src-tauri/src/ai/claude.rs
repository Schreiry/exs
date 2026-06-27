// Claude provider (Anthropic Messages API). Optional premium/high-quality path.
// Endpoint: POST https://api.anthropic.com/v1/messages
// Headers: x-api-key, anthropic-version: 2023-06-01.
// Model: claude-haiku-4-5-20251001 (fast, cheap, vision-capable). Override per
// install via the ANTHROPIC_MODEL env var (avoids code churn when aliases shift).

use crate::ai::provider::AiProvider;
use crate::ai::types::{AiAnswer, AiError, AiProviderKind, AiRequest, ImageInput, VisionMetadata};
use crate::ai::{prompts, vision};
use async_trait::async_trait;
use serde_json::json;

const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
const MODEL_ENV: &str = "ANTHROPIC_MODEL";

pub struct ClaudeProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeProvider {
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

    async fn message(&self, body: serde_json::Value) -> Result<String, AiError> {
        let resp = self
            .client
            .post(ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| AiError::Network(e.to_string()))?;
        if !status.is_success() {
            log::warn!("Claude returned HTTP {}", status);
            return Err(AiError::Provider(format!("HTTP {status}")));
        }
        let v: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;
        // content is an array of blocks; take the first text block.
        v["content"]
            .as_array()
            .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
            .and_then(|b| b["text"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AiError::Parse("no text block in Claude response".to_string()))
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn kind(&self) -> AiProviderKind {
        AiProviderKind::Claude
    }

    fn is_configured(&self) -> bool {
        !self.api_key.trim().is_empty()
    }

    async fn answer(&self, req: &AiRequest) -> Result<AiAnswer, AiError> {
        let body = json!({
            "model": self.model,
            "max_tokens": 800,
            "system": prompts::system_answer(&req.language),
            "messages": [
                {"role": "user", "content": prompts::answer_user_message(req)}
            ]
        });
        let text = self.message(body).await?;
        Ok(AiAnswer {
            text,
            language: req.language.clone(),
            provider: "claude".to_string(),
        })
    }

    async fn analyze_image(
        &self,
        image: &ImageInput,
        hint: Option<&str>,
    ) -> Result<VisionMetadata, AiError> {
        let body = json!({
            "model": self.model,
            "max_tokens": 700,
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "image", "source": {"type": "base64", "media_type": image.mime, "data": image.base64}},
                    {"type": "text", "text": prompts::vision_instruction(hint)}
                ]
            }]
        });
        let text = self.message(body).await?;
        vision::parse_vision_json(&text)
    }
}
