use crate::core::error::SchatError;
use crate::providers::base_client::HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common parser for OpenAI-Compatible streaming responses
pub fn openai_stream_parser(data: String) -> Result<Option<String>, SchatError> {
    let mut content = String::new();

    for line in data.lines() {
        if line.starts_with("data:") {
            let data = line[5..].trim();
            if data == "[DONE]" {
                return Ok(None);
            }

            let parsed: serde_json::Value = serde_json::from_str(data).map_err(|e| {
                SchatError::Serialization(format!("Failed to parse stream data: {}", e))
            })?;

            if let Some(choices) = parsed.get("choices").and_then(|c| c.as_array()) {
                if let Some(first_choice) = choices.first() {
                    if let Some(delta) = first_choice.get("delta") {
                        if let Some(text) = delta.get("content").and_then(|c| c.as_str()) {
                            content.push_str(text);
                        }
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
pub struct OpenAICompatibleProvider {
    client: HttpClient,
    pub model: String,
}

impl OpenAICompatibleProvider {
    pub fn new(
        base_url: String,
        api_key: String,
        model: String,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Self {
        // Use Bearer token authentication
        let auth_header = Some(("Authorization".to_string(), format!("Bearer {}", api_key)));

        Self {
            client: HttpClient::new(base_url, auth_header, extra_headers),
            model,
        }
    }

    pub async fn get_response(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<String, SchatError> {
        let req_messages: Vec<ChatCompletionMessage> = messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: match m.role {
                    crate::providers::Role::System => "system".to_string(),
                    crate::providers::Role::User => "user".to_string(),
                    crate::providers::Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let payload = ChatCompletionRequest {
            model: self.model.clone(),
            messages: req_messages,
            stream: None,
        };

        let response = self.client.post("chat/completions", &payload).await?;

        let response_body: String = response.text().await?;
        let parsed: ChatCompletionResponse = serde_json::from_str(&response_body)?;

        if parsed.choices.is_empty() {
            return Err(SchatError::Api("No choices in API response".to_string()));
        }

        Ok(parsed.choices[0].message.content.trim().to_string())
    }

    pub async fn get_response_stream(
        &self,
        messages: &[crate::providers::Message],
    ) -> Result<futures::stream::BoxStream<'static, Result<String, SchatError>>, SchatError> {
        let req_messages: Vec<ChatCompletionMessage> = messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: match m.role {
                    crate::providers::Role::System => "system".to_string(),
                    crate::providers::Role::User => "user".to_string(),
                    crate::providers::Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let payload = ChatCompletionRequest {
            model: self.model.clone(),
            messages: req_messages,
            stream: Some(true),
        };

        let response = self.client.post("chat/completions", &payload).await?;

        let stream = self
            .client
            .stream_response(response, openai_stream_parser)
            .await?;

        Ok(stream)
    }
}
