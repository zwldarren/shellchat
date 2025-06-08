use clap::Parser;
use config::{Config, Provider, ProviderConfig};

mod cli;
mod config;
mod display;
mod executor;
mod providers;
mod system;
mod utils;

use crate::cli::Args;
use crate::executor::execute_command;
use crate::providers::{
    LLMProvider, openai::OpenAIProvider, openrouter::OpenRouterProvider, process_response,
};
use crate::system::SystemInfo;

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
    let system_info = SystemInfo::new();

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
        handle_shell_mode(&args, &config, provider, &model, &system_info).await?;
    } else {
        handle_chat_mode(&args, provider, &model).await?;
    }

    Ok(())
}

async fn handle_shell_mode(
    args: &Args,
    config: &Config,
    provider: Box<dyn LLMProvider>,
    model: &str,
    system_info: &SystemInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create enhanced system prompt
    let prompt = SYSTEM_PROMPT_FOR_SHELL
        .replace("{shell}", &system_info.shell_path)
        .replace("{os_info}", &system_info.os_info);

    let raw_response = provider.get_response(&prompt, &args.query, model).await?;
    let command = process_response(&raw_response);

    display::display_command(&command);

    if !args.yes && !config.auto_confirm {
        if !display::prompt_execution_confirmation() {
            display::display_execution_cancelled();
            return Ok(());
        }
    }

    execute_command(&command, system_info)?;
    Ok(())
}

async fn handle_chat_mode(
    args: &Args,
    provider: Box<dyn LLMProvider>,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = provider
        .get_response(SYSTEM_PROMPT_FOR_CHAT, &args.query, model)
        .await?;

    // Display AI response using TUI module
    display::display_response(response.as_str());

    Ok(())
}
