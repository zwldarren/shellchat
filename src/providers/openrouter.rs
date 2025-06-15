use super::openai_compatible::OpenAICompatibleProvider;
use crate::core::error::SchatError;
use std::collections::HashMap;

#[derive(Clone)]
pub struct OpenRouterProvider {
    inner: OpenAICompatibleProvider,
}

impl OpenRouterProvider {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let base_url = "https://openrouter.ai/api/v1".to_string();
        let api_key = api_key.unwrap_or_default();
        let mut extra_headers = HashMap::new();
        extra_headers.insert(
            "HTTP-Referer".to_string(),
            "https://github.com/zwldarren/shellchat".to_string(),
        );
        extra_headers.insert("X-Title".to_string(), "ShellChat".to_string());

        Self {
            inner: OpenAICompatibleProvider::new(base_url, api_key, model, Some(extra_headers)),
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>, model: String) -> Self {
        let api_key = api_key.unwrap_or_default();
        let mut extra_headers = HashMap::new();
        extra_headers.insert(
            "HTTP-Referer".to_string(),
            "https://github.com/zwldarren/shellchat".to_string(),
        );
        extra_headers.insert("X-Title".to_string(), "ShellChat".to_string());

        Self {
            inner: OpenAICompatibleProvider::new(endpoint, api_key, model, Some(extra_headers)),
        }
    }
}

#[async_trait::async_trait]
impl super::LLMProvider for OpenRouterProvider {
    fn clone_provider(&self) -> Box<dyn super::LLMProvider> {
        Box::new(self.clone())
    }

    async fn get_response(&self, messages: &[super::Message]) -> Result<String, SchatError> {
        self.inner.get_response(messages).await
    }

    async fn get_response_stream(
        &self,
        messages: &[super::Message],
    ) -> Result<futures::stream::BoxStream<'static, Result<String, SchatError>>, SchatError> {
        self.inner.get_response_stream(messages).await
    }

    fn set_model(&mut self, model: &str) {
        let mut new_inner = self.inner.clone();
        new_inner.model = model.to_string();
        self.inner = new_inner;
    }
}
