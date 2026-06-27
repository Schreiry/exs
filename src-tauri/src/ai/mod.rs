// AI Gateway — провайдер-агностичный слой.
//
//   command ──▶ build_router(selected) ──▶ AiRouter ──▶ [OpenAI | Gemini | Claude | Mock]
//
// Команды строят роутер из выбранного провайдера (local_config) и доступных
// ключей (secrets/keyring). Если реальные провайдеры не настроены — используется
// Mock, чтобы UI работал в dev без ключей.

pub mod claude;
pub mod gemini;
pub mod localization;
pub mod mock;
pub mod openai;
pub mod prompts;
pub mod provider;
pub mod router;
pub mod secrets;
pub mod types;
pub mod vision;

use provider::AiProvider;
use types::{AiProviderKind, ProviderStatus};

/// Real (non-mock) providers in fixed fallback order.
const REAL_PROVIDERS: [AiProviderKind; 3] = [
    AiProviderKind::OpenAi,
    AiProviderKind::Gemini,
    AiProviderKind::Claude,
];

/// Shared HTTP client with a sane timeout (security: network calls must time out).
pub(crate) fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Instantiate a configured real provider (None if no key available).
fn make_real(kind: AiProviderKind) -> Option<Box<dyn AiProvider>> {
    let key = secrets::load_key(kind)?;
    let p: Box<dyn AiProvider> = match kind {
        AiProviderKind::OpenAi => Box::new(openai::OpenAiProvider::new(key, None)),
        AiProviderKind::Gemini => Box::new(gemini::GeminiProvider::new(key, None)),
        AiProviderKind::Claude => Box::new(claude::ClaudeProvider::new(key, None)),
        AiProviderKind::Mock => return Some(Box::new(mock::MockProvider)),
    };
    Some(p)
}

/// Build a router for the selected provider: selected first, then the other
/// configured real providers as fallbacks. Falls back to Mock if nothing real
/// is configured (so the void interface still works in dev).
pub fn build_router(selected: AiProviderKind) -> router::AiRouter {
    let mut providers: Vec<Box<dyn AiProvider>> = Vec::new();

    if selected != AiProviderKind::Mock {
        if let Some(p) = make_real(selected) {
            providers.push(p);
        }
    }
    for kind in REAL_PROVIDERS {
        if kind != selected {
            if let Some(p) = make_real(kind) {
                providers.push(p);
            }
        }
    }

    if providers.is_empty() {
        providers.push(Box::new(mock::MockProvider));
    }
    router::AiRouter::new(providers)
}

/// Status of every provider for the Settings UI (configured? primary?).
pub fn provider_statuses(selected: AiProviderKind) -> Vec<ProviderStatus> {
    let mut out: Vec<ProviderStatus> = REAL_PROVIDERS
        .iter()
        .map(|k| ProviderStatus {
            provider: k.as_str().to_string(),
            configured: secrets::has_key(*k),
            is_primary: *k == selected,
        })
        .collect();
    out.push(ProviderStatus {
        provider: "mock".to_string(),
        configured: true,
        is_primary: selected == AiProviderKind::Mock,
    });
    out
}
