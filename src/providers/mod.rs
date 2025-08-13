use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod openai;
pub mod claude;
pub mod gemini;

pub use openai::OpenAIProvider;
pub use claude::ClaudeProvider;
pub use gemini::GeminiProvider;

use crate::config::{Config, ProviderType};

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network timeout: {0}")]
    TimeoutError(String),
    
    #[error("Unknown provider error: {0}")]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub max_tokens: Option<u32>,
    pub supports_system_prompt: bool,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_command(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, ProviderError>;
    
    fn validate_config(&self, config: &Config) -> Result<(), ProviderError>;
    
    fn get_model_info(&self) -> ModelInfo;
    
    fn get_provider_name(&self) -> &'static str;
}

pub fn create_provider(config: &Config) -> Result<Box<dyn AIProvider>, ProviderError> {
    match config.provider_type {
        ProviderType::OpenAI | ProviderType::Custom => {
            let provider = OpenAIProvider::new(config)?;
            Ok(Box::new(provider))
        }
        ProviderType::Claude => {
            let provider = ClaudeProvider::new(config)?;
            Ok(Box::new(provider))
        }
        ProviderType::Gemini => {
            let provider = GeminiProvider::new(config)?;
            Ok(Box::new(provider))
        }
    }
}

pub fn get_default_model_for_provider(provider_type: &ProviderType) -> &'static str {
    match provider_type {
        ProviderType::OpenAI | ProviderType::Custom => "gpt-4o",
        ProviderType::Claude => "claude-3-5-sonnet-20241022",
        ProviderType::Gemini => "gemini-1.5-pro",
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
    
    pub struct MockProvider {
        responses: Mutex<VecDeque<Result<String, ProviderError>>>,
        model_info: ModelInfo,
    }
    
    impl MockProvider {
        pub fn new() -> Self {
            Self {
                responses: Mutex::new(VecDeque::new()),
                model_info: ModelInfo {
                    name: "mock-model".to_string(),
                    provider: "mock".to_string(),
                    max_tokens: Some(1000),
                    supports_system_prompt: true,
                },
            }
        }
        
        pub fn with_response(response: String) -> Self {
            let mut provider = Self::new();
            provider.add_response(Ok(response));
            provider
        }
        
        pub fn with_error(error: ProviderError) -> Self {
            let mut provider = Self::new();
            provider.add_response(Err(error));
            provider
        }
        
        pub fn add_response(&mut self, response: Result<String, ProviderError>) {
            self.responses.lock().unwrap().push_back(response);
        }
    }
    
    #[async_trait]
    impl AIProvider for MockProvider {
        async fn generate_command(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
        ) -> Result<String, ProviderError> {
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or_else(|| Ok("ls -la".to_string()))
        }
        
        fn validate_config(&self, _config: &Config) -> Result<(), ProviderError> {
            Ok(())
        }
        
        fn get_model_info(&self) -> ModelInfo {
            self.model_info.clone()
        }
        
        fn get_provider_name(&self) -> &'static str {
            "mock"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ProviderType};

    #[test]
    fn test_get_default_model_for_provider() {
        assert_eq!(get_default_model_for_provider(&ProviderType::OpenAI), "gpt-4o");
        assert_eq!(get_default_model_for_provider(&ProviderType::Claude), "claude-3-5-sonnet-20241022");
        assert_eq!(get_default_model_for_provider(&ProviderType::Gemini), "gemini-1.5-pro");
        assert_eq!(get_default_model_for_provider(&ProviderType::Custom), "gpt-4o");
    }

    #[test]
    fn test_provider_error_display() {
        let error = ProviderError::ApiError {
            status_code: 401,
            message: "Unauthorized".to_string(),
        };
        assert_eq!(error.to_string(), "API error: 401 - Unauthorized");
        
        let error = ProviderError::AuthenticationError("Invalid API key".to_string());
        assert_eq!(error.to_string(), "Authentication failed: Invalid API key");
    }

    #[tokio::test]
    async fn test_mock_provider() {
        use test_utils::MockProvider;
        
        let provider = MockProvider::with_response("echo 'test'".to_string());
        let result = provider.generate_command("system", "user").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "echo 'test'");
        
        let provider = MockProvider::with_error(
            ProviderError::AuthenticationError("Test error".to_string())
        );
        let result = provider.generate_command("system", "user").await;
        assert!(result.is_err());
    }
}