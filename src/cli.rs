use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Natural language query to convert to shell command
    pub query: String,

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
