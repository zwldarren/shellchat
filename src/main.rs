use clap::Parser;
use dotenv::dotenv;
use std::io;

mod cli;
mod executor;
mod providers;

use crate::cli::Args;
use crate::executor::execute_command;
use crate::providers::{LLMProvider, openai::OpenAIProvider};

const SYSTEM_PROMPT_FOR_SHELL: &str = "Convert this to a single bash command: ";
const SYSTEM_PROMPT_FOR_CHAT: &str = "You are a helpful assistant. Your response is limit to 100 words max. \
     Respond directly to the query without additional explanation unless asked.";

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

    let response = if args.shell {
        provider
            .get_response(&SYSTEM_PROMPT_FOR_SHELL, &args.query, &model)
            .await?
    } else {
        provider
            .get_response(SYSTEM_PROMPT_FOR_CHAT, &args.query, &model)
            .await?
    };

    if !args.shell {
        println!("\nAI Response:\n{}\n", response);
        return Ok(());
    }

    let command = response;
    println!("Generated command: \n{}\n", command);

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
