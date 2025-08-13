# sh-aid - Shell AI Assistant

> Convert natural language descriptions into shell commands using AI

sh-aid is a lightweight command-line tool that helps developers generate shell commands from natural language descriptions. Built in Rust for performance and reliability, it supports multiple AI providers through the [Rig framework](https://docs.rig.rs/).

## Features

- **Multi-provider AI support**: OpenAI, Claude, Gemini, and more
- **Context-aware**: Considers your OS, shell, and current directory
- **Fast & lightweight**: Single binary, sub-second startup
- **Secure**: Safe API key handling and command validation
- **Cross-platform**: Works on Linux, macOS, and Windows

## Quick Start

```bash
# Install (coming soon)
cargo install sh-aid

# Configure your API key
export OPENAI_API_KEY="your-api-key"

# Generate commands
sh-aid "list all files modified in the last 7 days"
# Output: find . -type f -mtime -7

sh-aid "compress all .txt files in current directory"
# Output: tar -czf text_files.tar.gz *.txt
```

## Supported Providers

- **OpenAI**: GPT-4, GPT-3.5-turbo
- **Anthropic**: Claude-3.5-sonnet, Claude-3-haiku  
- **Google**: Gemini-1.5-pro, Gemini-1.5-flash
- **Extensible**: Easy to add new Rig-supported providers

## Configuration

sh-aid uses a JSON configuration file stored in your platform's config directory:

```json
{
  "type": "OpenAI",
  "model": "gpt-4o",
  "apiKey": ""
}
```

API keys can be provided via:
- Configuration file
- Environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`)
- Command-line arguments (planned)

## Development Status

🚧 **Work in Progress** - This project is currently under active development.

- ✅ Project architecture and planning
- ✅ Configuration management
- ✅ System context gathering
- 🔄 Rig framework integration
- 🔄 Provider implementations
- ⏳ CLI interface
- ⏳ Testing and documentation

## Architecture

sh-aid is built with:
- **[Rig framework](https://docs.rig.rs/)** for LLM connectivity
- **Rust** for performance and safety
- **Cross-platform** design from the ground up

## Contributing

This project is in early development. Check out the [planning documents](./_docs/wip/) for technical details and roadmap.

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

*sh-aid is a Rust port inspired by shell AI assistant tools, designed for simplicity and performance.*