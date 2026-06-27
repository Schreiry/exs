// OpenAI provider. Использует Chat Completions API (стабильный, vision + JSON mode).
// NB: задание упоминает Responses API как primary — Chat Completions выбран для
// надёжности и предсказуемой формы ответа; переезд на /v1/responses — next step
// (эндпоинт/тело инкапсулированы здесь).
// Model: gpt-5-mini (cheapest GPT-5.x vision-capable tier as of writing; OpenAI
// aliases shift frequently — override per install via the OPENAI_MODEL env var
// rather than editing this constant).

use crate::ai::provider::AiProvider;
use crate::ai::types::{AiAnswer, AiError, AiProviderKind, AiRequest, ImageInput, VisionMetadata};
use crate::ai::{prompts, vision};
use async_trait::async_trait;
use serde_json::json;

const ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-5-mini";
const MODEL_ENV: &str = "OPENAI_MODEL";

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
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

    async fn chat(&self, body: serde_json::Value) -> Result<String, AiError> {
        let resp = self
            .client
            .post(ENDPOINT)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| AiError::Network(e.to_string()))?;
        if !status.is_success() {
            // Не логируем тело с возможными деталями ключа — только статус.
            log::warn!("OpenAI returned HTTP {}", status);
            return Err(AiError::Provider(format!("HTTP {status}")));
        }
        let v: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;
        v["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AiError::Parse("missing choices[0].message.content".to_string()))
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn kind(&self) -> AiProviderKind {
        AiProviderKind::OpenAi
    }

    fn is_configured(&self) -> bool {
        !self.api_key.trim().is_empty()
    }

    async fn answer(&self, req: &AiRequest) -> Result<AiAnswer, AiError> {
        let body = json!({
            "model": self.model,
            "max_tokens": 700,
            "messages": [
                {"role": "system", "content": prompts::system_answer(&req.language)},
                {"role": "user", "content": prompts::answer_user_message(req)}
            ]
        });
        let text = self.chat(body).await?;
        Ok(AiAnswer {
            text,
            language: req.language.clone(),
            provider: "openai".to_string(),
        })
    }

    async fn analyze_image(
        &self,
        image: &ImageInput,
        hint: Option<&str>,
    ) -> Result<VisionMetadata, AiError> {
        let data_url = format!("data:{};base64,{}", image.mime, image.base64);
        let body = json!({
            "model": self.model,
            "max_tokens": 600,
            "response_format": {"type": "json_object"},
            "messages": [{
                "role": "user",
                "content": [
                    {"type": "text", "text": prompts::vision_instruction(hint)},
                    {"type": "image_url", "image_url": {"url": data_url}}
                ]
            }]
        });
        let text = self.chat(body).await?;
        vision::parse_vision_json(&text)
    }
}
