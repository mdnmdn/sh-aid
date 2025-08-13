use async_trait::async_trait;

use super::{AIProvider, ModelInfo, ProviderError};
use crate::config::Config;

pub struct ClaudeProvider;

impl ClaudeProvider {
    pub fn new(_config: &Config) -> Result<Self, ProviderError> {
        // This is a placeholder implementation.
        Err(ProviderError::ConfigError(
            "Claude provider is not yet implemented.".to_string(),
        ))
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn generate_command(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
    ) -> Result<String, ProviderError> {
        Err(ProviderError::Unknown(
            "Claude provider is not yet implemented.".to_string(),
        ))
    }

    fn validate_config(&self, _config: &Config) -> Result<(), ProviderError> {
        Ok(())
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "claude-3-5-sonnet-20241022".to_string(),
            provider: "Claude".to_string(),
            max_tokens: Some(4096),
            supports_system_prompt: true,
        }
    }

    fn get_provider_name(&self) -> &'static str {
        "Claude"
    }
}
