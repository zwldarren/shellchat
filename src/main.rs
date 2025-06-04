use clap::Parser;
use dotenv::dotenv;
use std::fs;
use std::io;
use yaml_rust2::YamlLoader;

mod cli;
mod executor;
mod providers;

use crate::cli::Args;
use crate::executor::execute_command;
use crate::providers::{LLMProvider, openai::OpenAIProvider, openrouter::OpenRouterProvider, process_response};

const SYSTEM_PROMPT_FOR_SHELL: &str = "Convert this to a single bash command: ";
const SYSTEM_PROMPT_FOR_CHAT: &str = "You are a helpful assistant. Answer the following question in a concise manner: ";

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
struct ProviderConfig {
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Default)]
struct Config {
    active_provider: Option<String>,
    providers: std::collections::HashMap<String, ProviderConfig>,
}

fn load_config() -> Config {
    let config_path = "config.yaml";

    if let Ok(contents) = fs::read_to_string(config_path) {
        if let Ok(docs) = YamlLoader::load_from_str(&contents) {
            if let Some(doc) = docs.first() {
                let mut providers = std::collections::HashMap::new();
                if let Some(providers_yml) = doc["providers"].as_hash() {
                    for (name, config) in providers_yml {
                        if let Some(name_str) = name.as_str() {
                            providers.insert(
                                name_str.to_string(),
                                ProviderConfig {
                                    api_key: config["api_key"].as_str().map(|s| s.to_string()),
                                    base_url: config["base_url"].as_str().map(|s| s.to_string()),
                                    model: config["model"].as_str().map(|s| s.to_string()),
                                },
                            );
                        }
                    }
                }

                return Config {
                    active_provider: doc["active_provider"].as_str().map(|s| s.to_string()),
                    providers,
                };
            }
        }
    }

    Config::default()
}

fn merge_config_with_args(config: Config, args: &Args) -> (String, String, String, Option<String>) {
    let provider_name = args
        .provider
        .clone()
        .or(config.active_provider)
        .unwrap_or_else(|| "openai".to_string());

    let default_provider_config = ProviderConfig::default();
    let provider_config = config
        .providers
        .get(&provider_name)
        .unwrap_or(&default_provider_config);

    let base_url = args
        .base_url
        .clone()
        .or(provider_config.base_url.clone())
        .unwrap_or_else(|| {
            match provider_name.as_str() {
                "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                _ => "https://api.openai.com/v1".to_string(),
            }
        });

    let model = args
        .model
        .clone()
        .or(provider_config.model.clone())
        .unwrap_or_else(|| "gpt-4.1-mini".to_string());

    let api_key = provider_config.api_key.clone();

    (provider_name, base_url, model, api_key)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let args = Args::parse();
    let config = load_config();
    let (provider_name, base_url, model, api_key) = merge_config_with_args(config, &args);

    let provider: Box<dyn LLMProvider> = match provider_name.as_str() {
        "openai" => {
            if base_url == "https://api.openai.com/v1" {
                Box::new(OpenAIProvider::new(api_key))
            } else {
                Box::new(OpenAIProvider::with_endpoint(base_url.clone(), api_key))
            }
        }
        "openrouter" => {
            if base_url == "https://openrouter.ai/api/v1" {
                Box::new(OpenRouterProvider::new(api_key))
            } else {
                Box::new(OpenRouterProvider::with_endpoint(base_url.clone(), api_key))
            }
        }
        _ => return Err(format!("Unsupported provider: {}", provider_name).into()),
    };

    if args.shell {
        let raw_response = provider
            .get_response(&SYSTEM_PROMPT_FOR_SHELL, &args.query, &model)
            .await?;
        let command = process_response(&raw_response);
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
    } else {
        let response = provider
            .get_response(SYSTEM_PROMPT_FOR_CHAT, &args.query, &model)
            .await?;
        println!("\nAI Response:\n{}\n", response);
    }

    Ok(())
}
