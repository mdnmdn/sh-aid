use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{AIProvider, ModelInfo, ProviderError};
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    error: Option<OpenAIError>,
}

#[derive(Debug, Deserialize)]
struct OpenAIError {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(config: &Config) -> Result<Self, ProviderError> {
        let api_key = config
            .get_api_key()
            .ok_or_else(|| ProviderError::ConfigError("API key is required".to_string()))?
            .to_string();

        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "API key cannot be empty".to_string(),
            ));
        }

        let base_url = config
            .get_base_url()
            .unwrap_or("https://api.openai.com")
            .to_string();

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                ProviderError::ConfigError(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            client,
            api_key,
            model: config.model.clone(),
            base_url,
        })
    }

    fn build_request(&self, system_prompt: &str, user_prompt: &str) -> OpenAIRequest {
        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ];

        OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(1024),
            temperature: Some(0.0), // Use deterministic responses for command generation
        }
    }

    fn parse_response(&self, response: OpenAIResponse) -> Result<String, ProviderError> {
        // Check for API error first
        if let Some(error) = response.error {
            return match error.error_type.as_str() {
                "insufficient_quota" | "billing_hard_limit_reached" => {
                    Err(ProviderError::AuthenticationError(format!(
                        "Quota exceeded: {}",
                        error.message
                    )))
                }
                "invalid_api_key" | "invalid_request_error" => {
                    Err(ProviderError::AuthenticationError(error.message))
                }
                "rate_limit_exceeded" => Err(ProviderError::RateLimitError(error.message)),
                _ => Err(ProviderError::ApiError {
                    status_code: 400,
                    message: error.message,
                }),
            };
        }

        // Extract the command from the response
        let choice = response
            .choices
            .first()
            .ok_or_else(|| ProviderError::InvalidResponse("No choices in response".to_string()))?;

        let command = choice.message.content.trim();

        if command.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "Empty command response".to_string(),
            ));
        }

        Ok(command.to_string())
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_command(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, ProviderError> {
        let request = self.build_request(system_prompt, user_prompt);
        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::AuthenticationError(
                "Invalid API key or authentication failed".to_string(),
            ));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimitError(
                "Rate limit exceeded. Please try again later.".to_string(),
            ));
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::ApiError {
                status_code: status.as_u16(),
                message: error_text,
            });
        }

        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            ProviderError::InvalidResponse(format!("Failed to parse JSON response: {e}"))
        })?;

        self.parse_response(openai_response)
    }

    fn validate_config(&self, config: &Config) -> Result<(), ProviderError> {
        if config.get_api_key().is_none_or(|key| key.is_empty()) {
            return Err(ProviderError::ConfigError(
                "API key is required".to_string(),
            ));
        }

        if config.model.is_empty() {
            return Err(ProviderError::ConfigError(
                "Model name is required".to_string(),
            ));
        }

        // Validate base URL format if provided
        if let Some(base_url) = config.get_base_url() {
            if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                return Err(ProviderError::ConfigError(
                    "Base URL must start with http:// or https://".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.model.clone(),
            provider: "OpenAI".to_string(),
            max_tokens: Some(1024),
            supports_system_prompt: true,
        }
    }

    fn get_provider_name(&self) -> &'static str {
        "OpenAI"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ProviderType};

    fn create_test_config() -> Config {
        Config {
            provider_type: ProviderType::OpenAI,
            api_key: Some("test-key".to_string()),
            model: "gpt-4o".to_string(),
            base_url: None,
        }
    }

    #[test]
    fn test_openai_provider_creation() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.api_key, "test-key");
        assert_eq!(provider.model, "gpt-4o");
        assert_eq!(provider.base_url, "https://api.openai.com");
    }

    #[test]
    fn test_openai_provider_with_custom_base_url() {
        let mut config = create_test_config();
        config.base_url = Some("https://custom.openai.com".to_string());

        let provider = OpenAIProvider::new(&config);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.base_url, "https://custom.openai.com");
    }

    #[test]
    fn test_openai_provider_missing_api_key() {
        let mut config = create_test_config();
        config.api_key = None;

        let provider = OpenAIProvider::new(&config);
        assert!(provider.is_err());

        if let Err(ProviderError::ConfigError(msg)) = provider {
            assert!(msg.contains("API key is required"));
        } else {
            panic!("Expected ConfigError");
        }
    }

    #[test]
    fn test_build_request() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config).unwrap();

        let request = provider.build_request("system prompt", "user prompt");

        assert_eq!(request.model, "gpt-4o");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[0].content, "system prompt");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[1].content, "user prompt");
        assert_eq!(request.max_tokens, Some(1024));
        assert_eq!(request.temperature, Some(0.0));
    }

    #[test]
    fn test_parse_successful_response() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config).unwrap();

        let response = OpenAIResponse {
            choices: vec![OpenAIChoice {
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: "ls -la".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            error: None,
        };

        let result = provider.parse_response(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ls -la");
    }

    #[test]
    fn test_parse_error_response() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config).unwrap();

        let response = OpenAIResponse {
            choices: vec![],
            error: Some(OpenAIError {
                message: "Invalid API key".to_string(),
                error_type: "invalid_api_key".to_string(),
                code: None,
            }),
        };

        let result = provider.parse_response(response);
        assert!(result.is_err());

        if let Err(ProviderError::AuthenticationError(msg)) = result {
            assert_eq!(msg, "Invalid API key");
        } else {
            panic!("Expected AuthenticationError");
        }
    }

    #[test]
    fn test_validate_config() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config).unwrap();

        assert!(provider.validate_config(&config).is_ok());

        // Test missing API key
        let mut invalid_config = config.clone();
        invalid_config.api_key = None;
        assert!(provider.validate_config(&invalid_config).is_err());

        // Test empty model
        let mut invalid_config = config.clone();
        invalid_config.model = String::new();
        assert!(provider.validate_config(&invalid_config).is_err());

        // Test invalid base URL
        let mut invalid_config = config;
        invalid_config.base_url = Some("invalid-url".to_string());
        assert!(provider.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_get_model_info() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config).unwrap();

        let model_info = provider.get_model_info();
        assert_eq!(model_info.name, "gpt-4o");
        assert_eq!(model_info.provider, "OpenAI");
        assert_eq!(model_info.max_tokens, Some(1024));
        assert!(model_info.supports_system_prompt);
    }

    #[test]
    fn test_get_provider_name() {
        let config = create_test_config();
        let provider = OpenAIProvider::new(&config).unwrap();

        assert_eq!(provider.get_provider_name(), "OpenAI");
    }
}
