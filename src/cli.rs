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

    /// AI provider to use [possible values: openai]
    #[arg(short, long, default_value = "openai")]
    pub provider: String,

    /// Model to use (provider-specific)
    #[arg(short, long)]
    pub model: Option<String>,
}
