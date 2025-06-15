use super::openai_style::OpenAIStyleProvider;
use crate::core::error::SchatError;

#[derive(Clone)]
pub struct OpenAIProvider {
    inner: OpenAIStyleProvider,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let base_url = "https://api.openai.com/v1".to_string();
        let api_key = api_key.unwrap_or_default();
        Self {
            inner: OpenAIStyleProvider::new(base_url, api_key, model, None),
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>, model: String) -> Self {
        let api_key = api_key.unwrap_or_default();
        Self {
            inner: OpenAIStyleProvider::new(endpoint, api_key, model, None),
        }
    }
}

#[async_trait::async_trait]
impl super::LLMProvider for OpenAIProvider {
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
        // Clone and replace inner struct
        let mut new_inner = self.inner.clone();
        new_inner.model = model.to_string();
        self.inner = new_inner;
    }
}
