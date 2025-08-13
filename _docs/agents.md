# Shaid - Shell AI Assistant

## Project Overview

Shaid is a Rust port of a TypeScript-based shell helper tool that converts natural language descriptions into CLI commands using various AI providers. The project aims to provide developers with an intelligent command-line assistant that understands context and generates appropriate shell commands.

## Original Implementation (shaid-js.ts)

The original TypeScript implementation provides:
- Multi-provider AI support (OpenAI, Claude, Gemini)
- Configurable API settings via JSON config file
- Environment context gathering (OS info, system specs, directory listing)
- Natural language to shell command conversion

## Rust Port Goals

The Rust implementation will maintain feature parity while providing:
- Better performance and memory efficiency using Rig framework
- Cross-platform compatibility
- Robust error handling with built-in retry mechanisms
- Type safety and memory safety
- Single binary distribution
- Simplified LLM integration through unified abstractions

## Core Features

### Configuration Management
- JSON-based configuration file stored in platform-specific config directory
- Support for multiple AI providers through Rig framework:
  - OpenAI (GPT-4, GPT-3.5-turbo, custom endpoints)
  - Anthropic Claude (Claude-3.5-sonnet, Claude-3-haiku)
  - Google Gemini (Gemini-1.5-pro, Gemini-1.5-flash)
  - Extensible to other Rig-supported providers (DeepSeek, Ollama, etc.)
- Environment variable fallbacks for API keys
- Default configuration creation on first run
- Configuration validation against Rig provider capabilities

### Context Gathering
The tool collects system context to improve command generation accuracy:
- Operating system information (type, release, platform, architecture)
- Runtime environment details
- Shell information
- Current working directory
- System resources (CPU, memory)
- Directory listing (`ls` output)

### AI Integration with Rig Framework
- Unified interface for all LLM providers through Rig agents
- Simplified agent creation and configuration
- Built-in connection pooling and optimization
- Automatic retry mechanisms for transient failures
- Structured prompts with system context integration
- Robust error handling with provider-specific error mapping

### Command Generation
- Natural language input processing
- Context-aware command suggestions
- Clean output (command only, no extra text)
- Proper character escaping

## Project Structure

```
shaid/
  src/
    main.rs              # Entry point and CLI handling
    config.rs            # Configuration management
    providers/           # Rig-based provider implementations
       mod.rs            # Rig integration and adapter layer
       openai.rs         # OpenAI Rig agent factory
       claude.rs         # Claude Rig agent factory
       gemini.rs         # Gemini Rig agent factory
    context.rs           # System context gathering
    lib.rs              # Command generation logic
  Cargo.toml              # Rust dependencies (including Rig)
  _docs/                  # Documentation directory
     agents.md            # LLM agent context documentation
     wip/                 # Work-in-progress planning docs
        implementation-roadmap.md
        architecture-design.md
        testing-strategy.md
        rig-integration-plan.md
  README.md              # Project documentation
```

## Technical Considerations

### Dependencies
- `rig-core` and `rig` with provider features for LLM connectivity
- `serde` and `serde_json` for configuration serialization
- `tokio` for async runtime (compatible with Rig's async architecture)
- `dirs` for cross-platform configuration directory handling
- `sysinfo` for system context gathering
- `clap` for CLI argument parsing
- Error handling with `anyhow` and `thiserror`
- `async-trait` for async trait implementations

### Error Handling
- Rig framework's built-in retry mechanisms and connection pooling
- Graceful degradation when system commands fail
- Clear error messages for configuration issues
- Provider-specific error mapping from Rig's error types
- Automatic fallback between providers when possible
- Input validation and sanitization

### Security
- Secure API key storage and handling
- Input sanitization to prevent command injection
- Safe command output without exposing sensitive information
- Rig framework's secure HTTPS communications and certificate validation

### Performance Optimizations with Rig
- Reduced binary size compared to multiple HTTP client dependencies
- Built-in connection pooling for API requests
- Optimized JSON serialization/deserialization
- Automatic request batching where supported by providers
- Memory-efficient agent lifecycle management

## Development Guidelines

- Follow Rust best practices and idioms
- keep it deadly simple "simplicity is the ultimate perfection"
- Identify the best format (also multiple) to manage the configuration (eg: cli, env, .config/sh-aid/config.toml)
- Comprehensive error handling and user feedback
- Unit and integration tests for critical functionality
- Documentation for public APIs and configuration options
- be pragmantic and brutally honest

### Pre-commit Checks
Before submitting any code, please run the following checks to ensure code quality and consistency:
- `cargo fmt` - To format the code.
- `cargo check` - To check for compilation errors.
- `cargo clippy` - To check for lints and stylistic issues.

Make sure all checks pass before requesting a review.

## Implementation Status

### Completed
- ✅ Project planning and architecture design
- ✅ Rig framework integration plan
- ✅ Configuration management system
- ✅ System context gathering
- ✅ Provider abstraction layer
- ✅ Cargo.toml with Rig dependencies

### In Progress
- 🔄 Rig-based provider implementations
- 🔄 Command generation logic
- 🔄 CLI interface development

### Planned
- ⏳ Comprehensive testing suite
- ⏳ Cross-platform compatibility verification
- ⏳ Performance benchmarking
- ⏳ Documentation completion

## Future Enhancements

- Interactive mode for command confirmation
- Command history and learning from user patterns
- Plugin system for additional Rig-supported providers (Ollama, DeepSeek, etc.)
- Shell integration and auto-completion
- Command explanation and safety warnings
- Streaming response support for real-time feedback
