use crate::core::error::SchatError;
use crate::providers::base_client::HttpClient;
use crate::providers::{LLMProvider, Message, Role};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Parser for Anthropic's streaming response
pub fn anthropic_stream_parser(data: String) -> Result<Option<String>, SchatError> {
    let mut content = String::new();
    for line in data.lines() {
        if line.starts_with("data:") {
            let data_json = line[5..].trim();
            if data_json.is_empty() {
                continue;
            }
            let parsed: Value = match serde_json::from_str(data_json) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if parsed["type"] == "content_block_delta" {
                if let Some(delta) = parsed.get("delta") {
                    if delta["type"] == "text_delta" {
                        if let Some(text) = delta.get("text").and_then(|t| t.as_str()) {
                            content.push_str(text);
                        }
                    }
                }
            } else if parsed["type"] == "error" {
                if let Some(error) = parsed.get("error") {
                    if let Some(message) = error.get("message").and_then(|m| m.as_str()) {
                        return Err(SchatError::Api(format!(
                            "Anthropic stream error: {}",
                            message
                        )));
                    }
                }
            }
        }
    }

    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}

#[derive(Clone)]
pub struct AnthropicProvider {
    client: HttpClient,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: Option<String>, model: String) -> Self {
        let base_url = "https://api.anthropic.com/v1".to_string();
        let api_key = api_key.unwrap_or_default();
        let mut extra_headers = HashMap::new();
        extra_headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        Self {
            client: HttpClient::new(
                base_url,
                Some(("x-api-key".to_string(), api_key)),
                Some(extra_headers),
            ),
            model,
        }
    }

    pub fn with_endpoint(endpoint: String, api_key: Option<String>, model: String) -> Self {
        let api_key = api_key.unwrap_or_default();
        let mut extra_headers = HashMap::new();
        extra_headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        Self {
            client: HttpClient::new(
                endpoint,
                Some(("x-api-key".to_string(), api_key)),
                Some(extra_headers),
            ),
            model,
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    fn clone_provider(&self) -> Box<dyn LLMProvider> {
        Box::new(self.clone())
    }

    async fn get_response(&self, messages: &[Message]) -> Result<String, SchatError> {
        let system_prompt = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        let user_messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => unreachable!(),
                },
                content: m.content.clone(),
            })
            .collect();

        let payload = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: user_messages,
            stream: Some(false),
            system: system_prompt,
        };

        let response = self.client.post("messages", &payload).await?;
        let response_body = response.text().await?;
        let parsed: AnthropicResponse = serde_json::from_str(&response_body)?;

        if let Some(content_block) = parsed.content.first() {
            Ok(content_block.text.clone())
        } else {
            Err(SchatError::Api("Empty response from Anthropic".to_string()))
        }
    }

    async fn get_response_stream(
        &self,
        messages: &[Message],
    ) -> Result<BoxStream<'static, Result<String, SchatError>>, SchatError> {
        let system_prompt = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        let user_messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => unreachable!(),
                },
                content: m.content.clone(),
            })
            .collect();

        let payload = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: user_messages,
            stream: Some(true),
            system: system_prompt,
        };

        let response = self.client.post("messages", &payload).await?;

        let stream = self
            .client
            .stream_response(response, anthropic_stream_parser)
            .await?;

        Ok(stream)
    }

    fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}
