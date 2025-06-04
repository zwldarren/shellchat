use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Natural language query to chat with AI
    pub query: String,

    /// Generate and execute shell command
    #[arg(short, long)]
    pub shell: bool,

    /// Auto-confirm command execution without prompt
    #[arg(short, long)]
    pub yes: bool,

    /// AI provider (e.g. "openai")
    #[arg(short, long, default_value = "openai")]
    pub provider: String,

    /// API Endpoint for the provider
    #[arg(short, long, default_value = "https://api.openai.com/v1")]
    pub base_url: String,

    /// Model name (e.g. "gpt-4.1-mini")
    #[arg(short, long, default_value = "gpt-4.1-mini")]
    pub model: String,
}
