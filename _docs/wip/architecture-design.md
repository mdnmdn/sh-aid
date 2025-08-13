# sh-aid Architecture Design

## System Overview

sh-aid is designed as a modular command-line tool with clear separation of concerns and a plugin-like architecture for AI providers.

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Interface │    │  Configuration  │    │ System Context │
│   (main.rs)     │    │   (config.rs)   │    │  (context.rs)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │ Command Generator│
                    │   (lib.rs)      │
                    └─────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Rig Framework │
                    │   Integration   │
                    │  (providers/    │
                    │   mod.rs)       │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Rig OpenAI     │    │  Rig Claude     │    │  Rig Gemini     │
│  Agent          │    │  Agent          │    │  Agent          │
│ (openai.rs)     │    │ (claude.rs)     │    │ (gemini.rs)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Module Design

### Core Modules

#### 1. CLI Interface (`main.rs`)
**Responsibilities:**
- Parse command-line arguments
- Handle user input validation
- Coordinate between other modules
- Format and display output
- Handle top-level error cases

**Key Functions:**
```rust
fn main() -> Result<(), Box<dyn std::error::Error>>
fn parse_arguments() -> CommandArgs
fn display_result(command: String)
```

#### 2. Configuration Management (`config.rs`)
**Responsibilities:**
- Load and parse JSON configuration files
- Handle default configuration creation
- Manage environment variable fallbacks
- Validate configuration settings

**Data Structures:**
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider_type: ProviderType,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ProviderType {
    OpenAI,
    Claude,
    Gemini,
    Custom,
}
```

#### 3. System Context (`context.rs`)
**Responsibilities:**
- Gather operating system information
- Collect system resource data
- Execute safe system commands
- Build context strings for AI prompts

**Key Functions:**
```rust
pub fn gather_system_info() -> SystemInfo
pub fn get_directory_listing() -> Result<String, ContextError>
pub fn build_environment_context() -> String
```

#### 4. Command Generator (`lib.rs`)
**Responsibilities:**
- Coordinate command generation workflow
- Build AI prompts with system context
- Handle provider selection and switching
- Process and validate AI responses

**Key Functions:**
```rust
pub async fn generate_command(
    config: &Config,
    description: &str,
    context: &SystemContext,
) -> Result<String, GenerationError>
```

### Provider Architecture with Rig Framework

#### Rig Integration Layer (`providers/mod.rs`)
**Design Pattern:** Adapter Pattern wrapping Rig agents for consistent interface

```rust
use rig::completion::Prompt;

pub struct RigProviderAdapter {
    agent: Box<dyn rig::Agent>,
    provider_type: ProviderType,
}

impl RigProviderAdapter {
    pub async fn generate_command(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, ProviderError> {
        let full_prompt = format!("{}\n\nUser: {}", system_prompt, user_prompt);
        let response = self.agent.prompt(&full_prompt).await?;
        Ok(response)
    }
}
```

#### Individual Provider Factories
Each provider creates Rig agents with:
- Provider-specific client configuration
- Model selection and parameters
- Authentication setup
- Rig's built-in error handling and retry logic

## Data Flow

### 1. Initialization Flow
```
CLI Args → Config Loading → System Context Gathering → Provider Selection
```

### 2. Command Generation Flow
```
User Input → Prompt Building → API Request → Response Processing → Command Output
```

### 3. Error Handling Flow
```
Error Occurrence → Error Classification → User-Friendly Message → Graceful Exit
```

## Error Handling Strategy

### Error Hierarchy
```rust
#[derive(Debug, thiserror::Error)]
pub enum sh-aidError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Context gathering error: {0}")]
    Context(#[from] ContextError),
    
    #[error("Command generation error: {0}")]
    Generation(String),
}
```

### Error Recovery Patterns
1. **Configuration Errors**: Create default config, prompt user for required values
2. **API Errors**: Retry with exponential backoff, fallback to alternative providers
3. **System Context Errors**: Graceful degradation with partial context
4. **Network Errors**: Clear error messages with troubleshooting hints

## Security Considerations

### API Key Management
- Store keys in platform-specific secure directories
- Never log API keys or include in error messages
- Support environment variable overrides
- Validate key format before API calls

### Command Safety
- Input sanitization to prevent injection attacks
- Output validation to ensure safe command execution
- No automatic command execution (output only)
- Clear warnings for potentially dangerous commands

### Network Security
- Use HTTPS for all API communications
- Implement certificate validation
- Timeout handling for network requests
- Rate limiting to prevent abuse

## Performance Optimization

### Async Design
- Non-blocking I/O for API calls
- Concurrent system context gathering
- Streaming response processing where possible

### Memory Management
- Minimize string allocations
- Use borrowed strings where possible
- Implement proper resource cleanup
- Cache configuration data

### Startup Performance
- Lazy loading of non-essential modules
- Fast configuration parsing
- Minimal system context gathering

## Testing Strategy

### Unit Testing
- Each module tested in isolation
- Mock implementations for external dependencies
- Property-based testing for critical functions
- Error condition coverage

### Integration Testing
- End-to-end workflow testing
- Real API integration tests (with test keys)
- Cross-platform compatibility tests
- Performance regression tests

### Test Structure
```
tests/
├── unit/
│   ├── config_tests.rs
│   ├── context_tests.rs
│   └── provider_tests.rs
├── integration/
│   ├── full_workflow_tests.rs
│   └── cross_platform_tests.rs
└── mocks/
    ├── mock_providers.rs
    └── mock_http_client.rs
```

## Deployment Considerations

### Build Configuration
- Release builds with optimizations
- Static linking where possible
- Cross-compilation for multiple targets
- Binary size optimization

### Distribution
- Single binary deployment
- Platform-specific installers
- Package manager integration (brew, apt, etc.)
- Auto-update mechanism consideration

This architecture provides a solid foundation for the sh-aid implementation while maintaining flexibility for future enhancements and ensuring robust error handling and security.