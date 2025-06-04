use clap::Parser;
use dotenv::dotenv;
use std::io;

mod cli;
mod executor;
mod providers;

use crate::cli::Args;
use crate::executor::execute_command;
use crate::providers::{LLMProvider, openai::OpenAIProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args = Args::parse();

    let provider: Box<dyn LLMProvider> = match args.provider.as_str() {
        "openai" => Box::new(OpenAIProvider),
        _ => return Err(format!("Unsupported provider: {}", args.provider).into()),
    };

    let model = args.model.unwrap_or_else(|| match args.provider.as_str() {
        "openai" => "gpt-4.1-mini".to_string(),
        _ => "default".to_string(),
    });

    let command = provider.get_shell_command(&args.query, &model).await?;
    println!("Generated command: ```bash\n{}\n```", command);

    if !args.yes {
        println!("Execute this command? [y/N]");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Command execution cancelled");
            return Ok(());
        }
    }

    execute_command(&command)?;
    Ok(())
}
