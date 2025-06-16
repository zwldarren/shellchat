# ShellChat

ShellChat is a Rust CLI tool that brings AI to your terminal, enabling natural language command generation and interactive chat with various LLM providers. It supports MCP Servers and multiple AI providers: OpenAI, DeepSeek, Anthropic, Gemini and OpenRouter. Inspired by [ShellGPT](https://github.com/TheR1D/shell_gpt), it's both a practical tool and a Rust learning project.


## Example Usage

Chat with AI in the terminal:

```bash
schat "What is the capital of France?"
```

Convert natural language to shell commands:

```bash
schat -s "List all files in the current directory"
```

Continuous chat mode:
```bash
schat --chat
```
Enter an interactive chat session where you can converse with the AI model continuously. This mode supports:

*   **Interactive Conversation**: Engage in multi-turn dialogues with the AI.
*   **Slash Commands**: Use special commands to manage your chat session:
    *   `/help`: Display a list of all available commands.
    *   `/clear`: Clear the current conversation history.
    *   `/model <name>`: Show or change the active LLM model.
    *   `/save <filename>`: Save the current conversation history to a file.
    *   `/load <filename>`: Load a conversation history from a file.
    *   `/list`: List all saved conversation history files.
    *   `/delete <filename>`: Delete a specific conversation history file.
    *   `/display <mode>`: Control the visibility of tool interactions (modes: `verbose`, `minimal`, `hidden`, `help`).
*   **Tool Calling with MCP Servers**: When configured, the AI can automatically use tools provided by [MCP Servers](#MCP-Servers-Configuration) to perform actions like searching the web or accessing external APIs.

Generate a commit message for git changes:
```bash
git diff | schat "Generate a commit message for the changes"
```

Summary a text file:
```bash
schat "Summarize the content of this file" < file.txt
```

## Installation

### Quick Install from GitHub Releases

Linux Installation:
```bash
# One-line install using curl
curl -sSL https://raw.githubusercontent.com/zwldarren/shellchat/main/scripts/install.sh | bash -s -- install

# Uninstall
curl -sSL https://raw.githubusercontent.com/zwldarren/shellchat/main/scripts/install.sh | bash -s -- uninstall
```

Windows Installation:
```powershell
# One-line install using irm
irm https://raw.githubusercontent.com/zwldarren/shellchat/main/scripts/install.ps1 -OutFile $env:TEMP\install.ps1; & $env:TEMP\install.ps1 install

# Uninstall
irm https://raw.githubusercontent.com/zwldarren/shellchat/main/scripts/install.ps1 -OutFile $env:TEMP\install.ps1; & $env:TEMP\install.ps1 uninstall
```
## Configuration

The configuration file is located at `~/.schat/config.yaml`.

## Configuration Options

### Global Settings
```yaml
# Currently active LLM provider (must match a provider key below)
active_provider: openai

# Whether to auto-confirm shell command execution (true/false)
auto_confirm: false
```

### LLM Providers Configuration
Configure one or more LLM providers. All providers support these common parameters:
- `api_key`: Your API key for the provider
- `base_url`: API endpoint URL
- `model`: Model name to use

```yaml
providers:
  # OpenAI compatible API
  openai:
    api_key: your_openai_api_key_here
    base_url: https://api.openai.com/v1
    model: gpt-4.1-mini  # Example: gpt-4, gpt-3.5-turbo
  
  # OpenRouter API (supports multiple providers)
  openrouter:
    api_key: your_openrouter_api_key_here
    model: google/gemini-2.0-flash-001
  
  # DeepSeek API
  deepseek:
    api_key: your_deepseek_api_key_here
    model: deepseek-chat
  
  # Google Gemini API
  gemini:
    api_key: your_gemini_api_key_here
    model: gemini-2.0-flash
  
  # Anthropic Claude API
  anthropic:
    api_key: your_anthropic_api_key_here
    model: claude-sonnet-4-20250514
```

### MCP Servers Configuration
Model Context Protocol (MCP) servers extend functionality with additional tools.

```yaml
mcp_servers:
  # Local stdio-based server
  - name: everything
    enabled: false  # Set to true to enable
    type: stdio     # Local process
    command: npx    # Command to run
    args: 
      - "-y"
      - "@modelcontextprotocol/server-everything"
  
  # Brave Search API server
  - name: brave-search
    type: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-brave-search"]
    envs:
      BRAVE_API_KEY: your_brave_api_key_here  # Required API key
  
  # HTTP-based server
  - name: StreamableHttp-server
    type: streamable-http
    url: https://mcp.example.com/mcp
  
  # SSE-based server
  - name: sse-server
    type: sse
    url: https://mcp.example.com/sse
```

### Supported Providers
Get API keys from these providers:
- [OpenAI](https://platform.openai.com/docs/overview)
- [DeepSeek](https://platform.deepseek.com/api_keys)
- [Anthropic](https://console.anthropic.com/settings/keys)
- [Gemini](https://aistudio.google.com/apikey)
- [OpenRouter](https://openrouter.ai/settings/keys)
