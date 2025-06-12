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

Continuous chat mode (type /help for help, /exit to exit):
```bash
schat --chat
```

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
