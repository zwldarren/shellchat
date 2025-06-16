use crate::core::error::SchatError;
use crate::providers::LLMProvider;
use async_trait::async_trait;
use futures::stream::BoxStream;

mod client;
mod types;

pub use client::GeminiClient;

#[derive(Clone)]
pub struct GeminiProvider {
    client: GeminiClient,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let base_url = "https://generativelanguage.googleapis.com".to_string();
        let api_key = api_key.unwrap_or_default();
        Self {
            client: GeminiClient::new(base_url, api_key, model.clone(), None),
            model,
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>, model: String) -> Self {
        let api_key = api_key.unwrap_or_default();
        Self {
            client: GeminiClient::new(endpoint, api_key, model.clone(), None),
            model,
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn clone_provider(&self) -> Box<dyn LLMProvider> {
        Box::new(self.clone())
    }

    async fn get_response(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<String, SchatError> {
        self.client.generate_content(messages).await
    }

    async fn get_response_stream(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<BoxStream<'static, Result<String, SchatError>>, SchatError> {
        self.client.generate_content_stream(messages).await
    }

    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
        self.client.model = model.to_string();
    }
}
