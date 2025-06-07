use clap::Parser;
use config::{Config, Provider, ProviderConfig};
use os_info;
use std::env;

mod cli;
mod config;
mod display;
mod executor;
mod providers;

use crate::cli::Args;
use crate::executor::execute_command;
use crate::providers::{
    LLMProvider, openai::OpenAIProvider, openrouter::OpenRouterProvider, process_response,
};

const SYSTEM_PROMPT_FOR_SHELL: &str = "Convert the natural language query to a single command that \
will work on the current system. Only output the bare command without any explanation or markdown \
formatting. Include any necessary flags to make the command compatible with the current shell and OS. \
The current shell is {shell} and the OS is {os_info}.";
const SYSTEM_PROMPT_FOR_CHAT: &str =
    "You are a helpful assistant. Answer the following question in a concise manner: ";

fn merge_config_with_args(
    config: &Config,
    args: &Args,
) -> (Provider, String, String, Option<String>) {
    let provider = args
        .provider
        .as_ref()
        .and_then(|p| Provider::from_str(p))
        .or(config.active_provider)
        .unwrap_or_default();

    let default_provider_config = ProviderConfig::default();
    let provider_config = config
        .providers
        .get(&provider)
        .unwrap_or(&default_provider_config);

    let base_url = args
        .base_url
        .clone()
        .or(provider_config.base_url.clone())
        .unwrap_or_else(|| provider.default_base_url().to_string());

    let model = args
        .model
        .clone()
        .or(provider_config.model.clone())
        .unwrap_or_else(|| "gpt-4.1-mini".to_string());

    let api_key = provider_config.api_key.clone();

    (provider, base_url, model, api_key)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::load();
    let (provider_enum, base_url, model, api_key) = merge_config_with_args(&config, &args);

    let provider: Box<dyn LLMProvider> = match provider_enum {
        Provider::OpenAI => {
            if base_url == provider_enum.default_base_url() {
                Box::new(OpenAIProvider::new(api_key))
            } else {
                Box::new(OpenAIProvider::with_endpoint(base_url.clone(), api_key))
            }
        }
        Provider::OpenRouter => {
            if base_url == provider_enum.default_base_url() {
                Box::new(OpenRouterProvider::new(api_key))
            } else {
                Box::new(OpenRouterProvider::with_endpoint(base_url.clone(), api_key))
            }
        }
    };

    if args.shell {
        // Get system information
        let shell = env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
        let os_info = os_info::get();
        let os_info_str = format!(
            "{} {} {}",
            os_info.os_type(),
            os_info.version(),
            os_info.bitness()
        );

        // Create enhanced system prompt
        let prompt = SYSTEM_PROMPT_FOR_SHELL
            .replace("{shell}", &shell)
            .replace("{os_info}", &os_info_str);

        let raw_response = provider.get_response(&prompt, &args.query, &model).await?;
        let command = process_response(&raw_response);

        // Display AI-generated command using TUI module
        display::display_command(&command);

        if !args.yes && !config.auto_confirm {
            let execute = display::prompt_execution_confirmation();

            if !execute {
                display::display_execution_cancelled();
                return Ok(());
            }
        }

        execute_command(&command)?;
    } else {
        let response = provider
            .get_response(SYSTEM_PROMPT_FOR_CHAT, &args.query, &model)
            .await?;

        // Display AI response using TUI module
        display::display_response(response.as_str());
    }

    Ok(())
}
