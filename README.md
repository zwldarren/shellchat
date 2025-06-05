# ShellChat

A Rust CLI tool that provides AI-powered shell command generation and chat functionality through various LLM providers.

This project aims to learn Rust language while building a practical tool. It is inspired by [ShellGPT](https://github.com/TheR1D/shell_gpt).


## Example Usage

Convert natural language to shell commands:

```bash
schat -s "List all files in the current directory"
```

Chat with AI in the terminal:

```bash
schat "What is the capital of France?"
```

## Configuration

The configuration file is located at `~/.config/shell_chat/config.yaml`.

Example configuration:
```yaml
active_provider: openai
auto_confirm: false

providers:
  openai:
    api_key: your_openai_api_key_here
    base_url: https://api.openai.com/v1
    model: gpt-4.1-mini
  
  openrouter:
    api_key: your_openrouter_api_key_here
    base_url: https://openrouter.ai/api/v1
    model: google/gemini-2.0-flash-001
```

