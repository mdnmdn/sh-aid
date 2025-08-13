use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderType {
    #[serde(rename = "OpenAI")]
    OpenAI,
    #[serde(rename = "Custom")]
    Custom,
    #[serde(rename = "Claude")]
    Claude,
    #[serde(rename = "Gemini")]
    Gemini,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::OpenAI
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            provider_type: ProviderType::OpenAI,
            api_key: None,
            model: "gpt-4o".to_string(),
            base_url: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            return Self::create_default_config(&config_path);
        }

        let config_content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let mut user_config: Config = serde_json::from_str(&config_content)
            .with_context(|| {
                format!(
                    "Failed to parse config file: {:?}. Please ensure it is valid JSON.",
                    config_path
                )
            })?;

        // Apply environment variable fallbacks
        if user_config.api_key.is_none() || user_config.api_key.as_ref().map_or(true, |s| s.is_empty()) {
            user_config.api_key = get_env_api_key(&user_config.provider_type);
        }

        Ok(user_config)
    }

    fn create_default_config(config_path: &PathBuf) -> Result<Config> {
        // Create the config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create config directory: {:?}. Please check your permissions.",
                    parent
                )
            })?;
        }

        let default_config = Config::default();
        
        // Create the config file with empty API key
        let config_for_file = Config {
            api_key: Some(String::new()),
            base_url: None,
            ..default_config.clone()
        };

        let config_json = serde_json::to_string_pretty(&config_for_file)
            .context("Failed to serialize default config")?;

        fs::write(config_path, config_json).with_context(|| {
            format!(
                "Failed to create config file: {:?}. Please check your permissions.",
                config_path
            )
        })?;

        // Return config with environment API key for this first run
        let mut config = default_config;
        config.api_key = get_env_api_key(&config.provider_type);
        
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_none() || self.api_key.as_ref().map_or(true, |s| s.is_empty()) {
            anyhow::bail!(
                "API key not found. Please provide an API key in your config file or set the appropriate environment variable."
            );
        }

        if self.model.is_empty() {
            anyhow::bail!("Model name cannot be empty");
        }

        Ok(())
    }

    pub fn get_api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub fn get_base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to determine config directory")?
        .join("uwu");
    
    Ok(config_dir.join("config.json"))
}

fn get_env_api_key(provider_type: &ProviderType) -> Option<String> {
    match provider_type {
        ProviderType::OpenAI | ProviderType::Custom => {
            std::env::var("OPENAI_API_KEY").ok()
        }
        ProviderType::Claude => {
            std::env::var("ANTHROPIC_API_KEY").ok()
        }
        ProviderType::Gemini => {
            std::env::var("GOOGLE_API_KEY").ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(matches!(config.provider_type, ProviderType::OpenAI));
        assert_eq!(config.model, "gpt-4o");
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Should fail without API key
        assert!(config.validate().is_err());
        
        // Should succeed with API key
        config.api_key = Some("test-key".to_string());
        assert!(config.validate().is_ok());
        
        // Should fail with empty model
        config.model = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_api_key() {
        // Test OpenAI
        env::set_var("OPENAI_API_KEY", "test-openai-key");
        assert_eq!(
            get_env_api_key(&ProviderType::OpenAI),
            Some("test-openai-key".to_string())
        );
        env::remove_var("OPENAI_API_KEY");

        // Test Claude
        env::set_var("ANTHROPIC_API_KEY", "test-claude-key");
        assert_eq!(
            get_env_api_key(&ProviderType::Claude),
            Some("test-claude-key".to_string())
        );
        env::remove_var("ANTHROPIC_API_KEY");

        // Test Gemini
        env::set_var("GOOGLE_API_KEY", "test-gemini-key");
        assert_eq!(
            get_env_api_key(&ProviderType::Gemini),
            Some("test-gemini-key".to_string())
        );
        env::remove_var("GOOGLE_API_KEY");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            provider_type: ProviderType::Claude,
            api_key: Some("test-key".to_string()),
            model: "claude-3-sonnet".to_string(),
            base_url: Some("https://api.anthropic.com".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.provider_type, ProviderType::Claude));
        assert_eq!(deserialized.api_key, Some("test-key".to_string()));
        assert_eq!(deserialized.model, "claude-3-sonnet");
        assert_eq!(deserialized.base_url, Some("https://api.anthropic.com".to_string()));
    }
}