use super::LLMProvider;
use crate::core::error::SchatError;
use crate::providers::base_client::BaseApiClient;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    stream: Option<bool>,
}

#[derive(Serialize)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Clone)]
pub struct OpenRouterProvider {
    client: BaseApiClient,
    model: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let endpoint = "https://openrouter.ai/api/v1".to_string();
        let mut extra_headers = std::collections::HashMap::new();
        extra_headers.insert(
            "HTTP-Referer".to_string(),
            "https://github.com/zwldarren/shellchat".to_string(),
        );
        extra_headers.insert("X-Title".to_string(), "ShellChat".to_string());

        let api_key = api_key.unwrap_or_default();
        Self {
            client: BaseApiClient::new(endpoint, api_key, Some(extra_headers)),
            model,
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>, model: String) -> Self {
        let mut extra_headers = std::collections::HashMap::new();
        extra_headers.insert(
            "HTTP-Referer".to_string(),
            "https://github.com/zwldarren/shellchat".to_string(),
        );
        extra_headers.insert("X-Title".to_string(), "ShellChat".to_string());

        let api_key = api_key.unwrap_or_default();
        Self {
            client: BaseApiClient::new(endpoint, api_key, Some(extra_headers)),
            model,
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenRouterProvider {
    fn clone_provider(&self) -> Box<dyn LLMProvider> {
        Box::new(self.clone())
    }

    async fn get_response(&self, messages: &[super::Message]) -> Result<String, SchatError> {
        let req_messages: Vec<ChatCompletionMessage> = messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: match m.role {
                    super::Role::System => "system".to_string(),
                    super::Role::User => "user".to_string(),
                    super::Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let payload = ChatCompletionRequest {
            model: self.model.clone(),
            messages: req_messages,
            stream: None,
        };

        let response = self
            .client
            .send_request("chat/completions", &payload)
            .await?;

        let response_body: String = response.text().await?;
        let parsed: ChatCompletionResponse = serde_json::from_str(&response_body)?;

        if parsed.choices.is_empty() {
            return Err(SchatError::Api("No choices in API response".to_string()));
        }

        let content = parsed.choices[0].message.content.trim().to_string();

        if content.is_empty() {
            return Err(SchatError::Api(
                "Empty command received from API".to_string(),
            ));
        }

        Ok(content)
    }

    async fn get_response_stream(
        &self,
        messages: &[super::Message],
    ) -> Result<futures::stream::BoxStream<'static, Result<String, SchatError>>, SchatError> {
        let req_messages: Vec<ChatCompletionMessage> = messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: match m.role {
                    super::Role::System => "system".to_string(),
                    super::Role::User => "user".to_string(),
                    super::Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let payload = ChatCompletionRequest {
            model: self.model.clone(),
            messages: req_messages,
            stream: Some(true),
        };

        let stream = self
            .client
            .get_response_stream("chat/completions", &payload)
            .await?;

        Ok(stream)
    }
    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}
