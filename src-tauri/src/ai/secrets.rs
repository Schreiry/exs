// Secure storage for provider API keys. Никогда не храним ключи в коде/конфиге.
// Приоритет: OS-native secure storage (keyring) → переменная окружения (для dev).
// Ключи НИКОГДА не логируются и не пишутся в audit_logs.

use crate::ai::types::AiProviderKind;
use keyring::Entry;

const SERVICE: &str = "com.exsul.app";

fn key_name(kind: AiProviderKind) -> String {
    format!("ai_api_key_{}", kind.as_str())
}

fn env_var(kind: AiProviderKind) -> &'static str {
    match kind {
        AiProviderKind::OpenAi => "OPENAI_API_KEY",
        AiProviderKind::Gemini => "GEMINI_API_KEY",
        AiProviderKind::Claude => "ANTHROPIC_API_KEY",
        AiProviderKind::Mock => "MOCK_API_KEY",
    }
}

/// Store an API key in OS-native secure storage.
pub fn store_key(kind: AiProviderKind, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, &key_name(kind)).map_err(|e| e.to_string())?;
    entry.set_password(value).map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete a stored API key.
pub fn delete_key(kind: AiProviderKind) -> Result<(), String> {
    let entry = Entry::new(SERVICE, &key_name(kind)).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        // Treating "not found" as success keeps delete idempotent.
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Load an API key: keyring first, then env var fallback (dev convenience).
pub fn load_key(kind: AiProviderKind) -> Option<String> {
    if let Ok(entry) = Entry::new(SERVICE, &key_name(kind)) {
        if let Ok(secret) = entry.get_password() {
            if !secret.trim().is_empty() {
                return Some(secret);
            }
        }
    }
    std::env::var(env_var(kind)).ok().filter(|s| !s.trim().is_empty())
}

/// Whether a key is available (keyring or env), without returning it.
pub fn has_key(kind: AiProviderKind) -> bool {
    load_key(kind).is_some()
}
