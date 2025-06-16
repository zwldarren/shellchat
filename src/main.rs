use crate::config::{Config, Provider, ProviderConfig};
use crate::providers::factory::ProviderFactory;
use clap::Parser;

mod app;
mod cli;
mod commands;
mod config;
mod core;
mod display;
mod input;
mod mcp;
mod providers;
mod system;

use crate::app::Application;
use crate::cli::parser::Args;
use crate::commands::create_command_registry;
use crate::core::error::SchatError;

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

    let default_config: ProviderConfig = ProviderConfig {
        base_url: None,
        model: None,
        api_key: None,
    };
    let provider_config = config.providers.get(&provider).unwrap_or(&default_config);

    let base_url = args
        .base_url
        .clone()
        .or_else(|| provider_config.base_url.clone())
        .unwrap_or_else(|| provider.default_base_url().to_string());

    let model = args
        .model
        .clone()
        .or_else(|| provider_config.model.clone())
        .unwrap_or_else(|| "gpt-4.1-mini".to_string());

    (provider, base_url, model, provider_config.api_key.clone())
}

#[tokio::main]
async fn main() -> Result<(), SchatError> {
    let args = Args::parse();
    let config = Config::load()?;
    let (provider_enum, base_url, model, api_key) = merge_config_with_args(&config, &args);

    let provider_factory = ProviderFactory::new();
    let provider_config = ProviderConfig {
        base_url: Some(base_url),
        model: Some(model),
        api_key,
    };

    let provider = provider_factory.create(&provider_enum, &provider_config)?;

    let command_dispatcher = create_command_registry();

    let mut app = Application::new(args, config, provider, command_dispatcher)?;

    app.run().await
}
