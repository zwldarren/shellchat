use super::LLMProvider;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::env;

pub struct OpenRouterProvider {
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

impl OpenRouterProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            endpoint: None,
            api_key,
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            endpoint: Some(endpoint),
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn get_response(
        &self,
        messages: &[super::Message],
        model: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => env::var("OPENROUTER_API_KEY").map_err(
                |_| "OPENROUTER_API_KEY must be set from config or environment variable",
            )?,
        };

        if api_key.trim().is_empty() {
            return Err("OPENROUTER_API_KEY cannot be empty".into());
        }

        // Build the client
        let mut client_builder = OpenAIClient::builder().with_api_key(api_key);

        // Set endpoint if explicitly provided, otherwise use default OpenRouter endpoint
        let endpoint = self
            .endpoint
            .as_ref()
            .map(|e| e.as_str())
            .unwrap_or("https://openrouter.ai/api/v1");
        client_builder = client_builder.with_endpoint(endpoint);

        let mut client = client_builder.build()?;

        // Convert messages to the required format
        let messages: Vec<chat_completion::ChatCompletionMessage> = messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    super::Role::System => chat_completion::MessageRole::system,
                    super::Role::User => chat_completion::MessageRole::user,
                    super::Role::Assistant => chat_completion::MessageRole::assistant,
                };
                chat_completion::ChatCompletionMessage {
                    role,
                    content: chat_completion::Content::Text(m.content.clone()),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }
            })
            .collect();

        // Create the chat completion request
        let req = ChatCompletionRequest::new(model.to_string(), messages);

        // Send the request
        let result = client.chat_completion(req).await?;

        if result.choices.is_empty() {
            return Err("No choices in API response".into());
        }

        let content = match &result.choices[0].message.content {
            Some(text) => text.trim(),
            None => return Err("No content in API response".into()),
        };

        let command = content.to_string();

        if command.is_empty() {
            return Err("Empty command received from API".into());
        }

        Ok(command)
    }
}
