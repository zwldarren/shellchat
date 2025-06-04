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

    /// AI provider (e.g. "openai") - defaults from config.yaml if not specified
    #[arg(short, long)]
    pub provider: Option<String>,

    /// API Endpoint for the provider - defaults from config.yaml if not specified
    #[arg(short, long)]
    pub base_url: Option<String>,

    /// Model name (e.g. "gpt-4.1-mini") - defaults from config.yaml if not specified
    #[arg(short, long)]
    pub model: Option<String>,
}
