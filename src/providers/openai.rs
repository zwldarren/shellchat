use super::LLMProvider;
use crate::error::SchatError;
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

pub struct OpenAIProvider {
    client: BaseApiClient,
}

impl OpenAIProvider {
    pub fn new(api_key: Option<String>) -> Self {
        let endpoint = "https://api.openai.com/v1".to_string();
        let api_key = api_key.unwrap_or_default();
        Self {
            client: BaseApiClient::new(endpoint, api_key, None),
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>) -> Self {
        let api_key = api_key.unwrap_or_default();
        Self {
            client: BaseApiClient::new(endpoint, api_key, None),
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    async fn get_response(
        &self,
        messages: &[super::Message],
        model: &str,
    ) -> Result<String, SchatError> {
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
            model: model.to_string(),
            messages: req_messages,
            stream: None,
        };

        let response = self
            .client
            .send_request("chat/completions", &payload)
            .await?;

        let response_body = response.text().await?;
        let parsed: ChatCompletionResponse = serde_json::from_str(&response_body)?;

        if parsed.choices.is_empty() {
            return Err("No choices in API response".into());
        }

        let content = parsed.choices[0].message.content.trim().to_string();

        if content.is_empty() {
            return Err("Empty command received from API".into());
        }

        Ok(content)
    }

    async fn get_response_stream(
        &self,
        messages: &[super::Message],
        model: &str,
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
            model: model.to_string(),
            messages: req_messages,
            stream: Some(true),
        };

        let stream = self
            .client
            .get_response_stream("chat/completions", &payload)
            .await?;

        Ok(stream)
    }
}
