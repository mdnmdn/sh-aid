use async_trait::async_trait;

use super::{AIProvider, ModelInfo, ProviderError};
use crate::config::Config;

pub struct GeminiProvider;

impl GeminiProvider {
    pub fn new(_config: &Config) -> Result<Self, ProviderError> {
        // This is a placeholder implementation.
        Err(ProviderError::ConfigError(
            "Gemini provider is not yet implemented.".to_string(),
        ))
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn generate_command(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
    ) -> Result<String, ProviderError> {
        Err(ProviderError::Unknown(
            "Gemini provider is not yet implemented.".to_string(),
        ))
    }

    fn validate_config(&self, _config: &Config) -> Result<(), ProviderError> {
        Ok(())
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "gemini-1.5-pro".to_string(),
            provider: "Gemini".to_string(),
            max_tokens: Some(8192),
            supports_system_prompt: true,
        }
    }

    fn get_provider_name(&self) -> &'static str {
        "Gemini"
    }
}
