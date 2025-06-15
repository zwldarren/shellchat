use crate::config::{Provider, ProviderConfig};
use crate::core::error::SchatError;
use crate::providers::{LLMProvider, openai::OpenAIProvider, openrouter::OpenRouterProvider};
use std::collections::HashMap;

type ProviderCreator =
    Box<dyn Fn(&ProviderConfig) -> Result<Box<dyn LLMProvider>, SchatError> + Send + Sync>;

pub struct ProviderFactory {
    creators: HashMap<Provider, ProviderCreator>,
}

impl ProviderFactory {
    pub fn new() -> Self {
        let mut creators = HashMap::new();

        creators.insert(
            Provider::OpenAI,
            Box::new(|config: &ProviderConfig| {
                let model = config
                    .model
                    .clone()
                    .unwrap_or_else(|| "gpt-4.1-mini".to_string());
                let provider = if let Some(base_url) = &config.base_url {
                    OpenAIProvider::with_endpoint(base_url.clone(), config.api_key.clone(), model)
                } else {
                    OpenAIProvider::new(config.api_key.clone(), model)
                };
                Ok(Box::new(provider) as Box<dyn LLMProvider>)
            }) as ProviderCreator,
        );

        creators.insert(
            Provider::OpenRouter,
            Box::new(|config: &ProviderConfig| {
                let model = config
                    .model
                    .clone()
                    .unwrap_or_else(|| "google/gemini-2.0-flash-001".to_string());
                let provider = if let Some(base_url) = &config.base_url {
                    OpenRouterProvider::with_endpoint(
                        base_url.clone(),
                        config.api_key.clone(),
                        model,
                    )
                } else {
                    OpenRouterProvider::new(config.api_key.clone(), model)
                };
                Ok(Box::new(provider) as Box<dyn LLMProvider>)
            }) as ProviderCreator,
        );

        Self { creators }
    }

    pub fn create(
        &self,
        provider: &Provider,
        config: &ProviderConfig,
    ) -> Result<Box<dyn LLMProvider>, SchatError> {
        self.creators
            .get(provider)
            .ok_or_else(|| SchatError::Config(format!("Provider not found: {:?}", provider)))
            .and_then(|creator| creator(config))
    }
}
