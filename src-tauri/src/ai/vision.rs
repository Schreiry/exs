// Vision helpers — превращают сырой текстовый ответ провайдера в типизированные
// VisionMetadata. Провайдеры (openai/gemini/claude) дергают parse_vision_json.

use crate::ai::prompts;
use crate::ai::types::{AiError, VisionMetadata};

/// Parse a provider's (possibly prose-wrapped) reply into VisionMetadata.
pub fn parse_vision_json(text: &str) -> Result<VisionMetadata, AiError> {
    let json = prompts::extract_json_object(text)
        .ok_or_else(|| AiError::Parse("no JSON object in vision response".to_string()))?;
    let meta: VisionMetadata =
        serde_json::from_str(json).map_err(|e| AiError::Parse(format!("vision JSON: {e}")))?;
    Ok(meta)
}
