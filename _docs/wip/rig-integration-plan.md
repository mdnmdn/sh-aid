# Rig Framework Integration Plan for sh-aid

## Overview

This document outlines the integration of the Rig framework (https://docs.rig.rs/) into the sh-aid project to simplify LLM connectivity and improve performance. Rig provides a lightweight, modular abstraction over multiple LLM providers with built-in error handling and optimization.

## Benefits of Using Rig

### Technical Advantages
- **Simplified Integration**: Single API for multiple LLM providers
- **Built-in Optimizations**: Performance optimizations and connection pooling
- **Error Handling**: Robust error handling with automatic retries
- **Type Safety**: Strong Rust types with compile-time guarantees
- **Lightweight**: Minimal overhead compared to direct API implementations
- **WebAssembly Support**: Can compile to WASM for browser deployment

### Supported Providers
- **OpenAI**: GPT-4, GPT-3.5-turbo, and other OpenAI models
- **Anthropic**: Claude family of models
- **Google**: Gemini models
- **Additional**: DeepSeek, Ollama, Perplexity, Hugging Face, XAI

## Architecture Changes

### Current Direct API Approach
```rust
// Custom HTTP client implementations
impl OpenAIProvider {
    async fn generate_command(&self, prompt: &str) -> Result<String, ProviderError> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&request)
            .send()
            .await?;
        // Manual response parsing...
    }
}
```

### New Rig-Based Approach
```rust
use rig::{completion::Prompt, providers::openai};

pub struct sh-aidAgent {
    agent: Box<dyn rig::Agent>,
    provider_type: ProviderType,
}

impl sh-aidAgent {
    pub fn new_openai(api_key: &str, model: &str) -> Result<Self, ProviderError> {
        let client = openai::Client::new(api_key);
        let agent = client.agent(model).build();
        Ok(Self {
            agent: Box::new(agent),
            provider_type: ProviderType::OpenAI,
        })
    }

    pub async fn generate_command(&self, prompt: &str) -> Result<String, ProviderError> {
        let response = self.agent.prompt(prompt).await?;
        Ok(response)
    }
}
```

## Updated Dependencies

### Cargo.toml Changes
```toml
[dependencies]
# Rig framework for LLM connectivity
rig-core = "0.3"

# Provider-specific Rig modules
rig-anthropic = "0.3"
rig-google = "0.3"  
rig-openai = "0.3"

# Core dependencies (reduced due to Rig handling HTTP/JSON)
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.5", features = ["derive"] }
dirs = "5.0"
anyhow = "1.0"
thiserror = "2.0"
sysinfo = "0.32"
async-trait = "0.1"

# Remove these (handled by Rig):
# reqwest = { version = "0.12", features = ["json"] }
```

## Implementation Plan

### Phase 1: Rig Framework Setup
1. **Update Cargo.toml** with Rig dependencies
2. **Create Rig adapter layer** in `src/providers/mod.rs`
3. **Define provider factory functions** for each LLM provider
4. **Update error types** to work with Rig's error system

### Phase 2: Provider Implementations
1. **OpenAI Provider** (`src/providers/openai.rs`)
   ```rust
   use rig::providers::openai;
   
   pub fn create_openai_agent(config: &Config) -> Result<Box<dyn rig::Agent>, ProviderError> {
       let client = openai::Client::new(&config.api_key.as_ref().unwrap());
       let mut agent_builder = client.agent(&config.model);
       
       if let Some(base_url) = &config.base_url {
           agent_builder = agent_builder.base_url(base_url);
       }
       
       Ok(Box::new(agent_builder.build()))
   }
   ```

2. **Claude Provider** (`src/providers/claude.rs`)
   ```rust
   use rig::providers::anthropic;
   
   pub fn create_claude_agent(config: &Config) -> Result<Box<dyn rig::Agent>, ProviderError> {
       let client = anthropic::Client::new(&config.api_key.as_ref().unwrap());
       let agent = client.agent(&config.model).build();
       Ok(Box::new(agent))
   }
   ```

3. **Gemini Provider** (`src/providers/gemini.rs`)
   ```rust
   use rig::providers::google;
   
   pub fn create_gemini_agent(config: &Config) -> Result<Box<dyn rig::Agent>, ProviderError> {
       let client = google::Client::new(&config.api_key.as_ref().unwrap());
       let agent = client.agent(&config.model).build();
       Ok(Box::new(agent))
   }
   ```

### Phase 3: Configuration Mapping
1. **Update Config struct** to support Rig-specific options
2. **Add model validation** against Rig's supported models
3. **Environment variable mapping** for different providers
4. **Default model selection** per provider

### Phase 4: Prompt Engineering
1. **System prompt integration** with Rig's prompt system
2. **Context formatting** for optimal model performance
3. **Response parsing** and validation
4. **Command extraction** from LLM responses

## Error Handling Integration

### Rig Error Mapping
```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Rig completion error: {0}")]
    RigError(#[from] rig::completion::CompletionError),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Model not available: {0}")]
    ModelError(String),
}

impl From<rig::completion::CompletionError> for ProviderError {
    fn from(err: rig::completion::CompletionError) -> Self {
        match err {
            rig::completion::CompletionError::ApiKeyMissing => {
                ProviderError::AuthenticationError("API key missing".to_string())
            }
            rig::completion::CompletionError::NetworkError(msg) => {
                ProviderError::ConfigError(format!("Network error: {}", msg))
            }
            _ => ProviderError::RigError(err),
        }
    }
}
```

## Testing Strategy Updates

### Mock Rig Agents
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rig::completion::Prompt;
    
    struct MockRigAgent {
        responses: Vec<String>,
        current_index: std::sync::atomic::AtomicUsize,
    }
    
    #[async_trait]
    impl rig::Agent for MockRigAgent {
        async fn prompt(&self, _prompt: &str) -> Result<String, rig::completion::CompletionError> {
            let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.responses.get(index)
                .unwrap_or(&"ls -la".to_string())
                .clone())
        }
    }
}
```

### Integration Tests
- **Provider switching** tests
- **Configuration validation** tests
- **Error handling** tests with Rig errors
- **Performance comparison** against direct API calls

## Migration Strategy

### Phase 1: Parallel Implementation
1. Keep existing provider implementations
2. Add Rig-based providers alongside
3. Feature flag to switch between implementations
4. Performance and reliability comparison

### Phase 2: Gradual Migration
1. Default to Rig providers for new installations
2. Provide migration path for existing configurations
3. Deprecation warnings for direct API implementations

### Phase 3: Full Migration
1. Remove direct API implementations
2. Simplify codebase using only Rig abstractions
3. Update documentation and examples

## Performance Considerations

### Expected Improvements
- **Reduced Binary Size**: Less HTTP client code
- **Better Connection Management**: Rig's built-in pooling
- **Automatic Retries**: Built-in retry logic for transient failures
- **Optimized Serialization**: Rig's optimized JSON handling

### Benchmarking Plan
- **Startup time** comparison
- **Request latency** measurements
- **Memory usage** profiling
- **Binary size** analysis

## Risk Assessment

### Low Risk
- **API Compatibility**: Rig provides stable abstractions
- **Documentation**: Well-documented framework
- **Community Support**: Active development and maintenance

### Medium Risk
- **Learning Curve**: Team needs to learn Rig patterns
- **Dependency Management**: Additional dependency to maintain
- **Feature Parity**: Ensuring all current features work with Rig

### Mitigation Strategies
- **Gradual adoption** with feature flags
- **Comprehensive testing** during migration
- **Fallback mechanisms** to direct API calls if needed

## Success Metrics

### Technical Metrics
- **Reduced Lines of Code**: 30-40% reduction in provider implementations
- **Improved Test Coverage**: Easier mocking and testing
- **Better Error Handling**: More consistent error reporting
- **Performance Gains**: Faster startup and response times

### User Experience Metrics
- **Reliability**: Fewer API-related failures
- **Compatibility**: Seamless migration for existing users
- **Feature Completeness**: All existing features preserved

This integration plan provides a clear path to modernize sh-aid's LLM connectivity while maintaining reliability and improving performance.