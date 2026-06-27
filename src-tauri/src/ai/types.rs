// Типы AI Gateway — провайдер-агностичный контракт. Эти структуры — единственный
// формат, который видят команды и фронтенд; конкретные провайдеры (OpenAI/Gemini/
// Claude) маппят их в свои HTTP-форматы внутри себя.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Which AI provider to use. Stored in local_config as a plain string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderKind {
    OpenAi,
    Gemini,
    Claude,
    Mock,
}

impl AiProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProviderKind::OpenAi => "openai",
            AiProviderKind::Gemini => "gemini",
            AiProviderKind::Claude => "claude",
            AiProviderKind::Mock => "mock",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "openai" => AiProviderKind::OpenAi,
            "gemini" => AiProviderKind::Gemini,
            "claude" => AiProviderKind::Claude,
            _ => AiProviderKind::Mock,
        }
    }
}

/// Minimal product data sent to the provider as grounding context. Мы НИКОГДА не
/// отправляем всю БД — только релевантные карточки (см. security в CLAUDE.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductContext {
    pub item_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub price: f64,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Explicitly selected local text supplied as untrusted reference material.
/// Content is bounded before it reaches a provider and must never be treated
/// as system instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub truncated: bool,
}

/// One image to analyze, base64-encoded (no data: prefix).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInput {
    pub mime: String,
    pub base64: String,
}

/// A text-answer request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub query: String,
    /// ru | ka | en
    pub language: String,
    #[serde(default)]
    pub context_items: Vec<ProductContext>,
    #[serde(default)]
    pub context_files: Vec<FileContext>,
}

/// A text answer from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnswer {
    pub text: String,
    pub language: String,
    pub provider: String,
}

/// Structured metadata extracted from a product photo. This is the typed JSON
/// the UI/DB rely on — NOT free text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisionMetadata {
    #[serde(default)]
    pub caption_ru: Option<String>,
    #[serde(default)]
    pub caption_ka: Option<String>,
    #[serde(default)]
    pub caption_en: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub visual_attributes: serde_json::Value,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub confidence: f64,
}

/// Provider error. Serializes to a stable string for Tauri commands; secrets are
/// never included in the message (см. security rule #8).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message")]
pub enum AiError {
    /// No provider is configured / no API key available.
    NotConfigured(String),
    /// Network/timeout failure.
    Network(String),
    /// Provider returned an error response.
    Provider(String),
    /// Could not parse the provider response into our typed shape.
    Parse(String),
    /// Provider doesn't support the requested capability (e.g. vision).
    Unsupported(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::NotConfigured(m) => write!(f, "ai not configured: {m}"),
            AiError::Network(m) => write!(f, "ai network error: {m}"),
            AiError::Provider(m) => write!(f, "ai provider error: {m}"),
            AiError::Parse(m) => write!(f, "ai parse error: {m}"),
            AiError::Unsupported(m) => write!(f, "ai unsupported: {m}"),
        }
    }
}

impl std::error::Error for AiError {}

impl From<AiError> for String {
    fn from(e: AiError) -> Self {
        e.to_string()
    }
}

/// Provider availability snapshot for the Settings UI.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatus {
    pub provider: String,
    pub configured: bool,
    pub is_primary: bool,
}
