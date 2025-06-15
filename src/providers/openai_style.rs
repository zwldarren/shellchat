use crate::core::error::SchatError;
use crate::providers::base_client::BaseApiClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct OpenAIStyleProvider {
    client: BaseApiClient,
    pub model: String,
}

impl OpenAIStyleProvider {
    pub fn new(
        base_url: String,
        api_key: String,
        model: String,
        extra_headers: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            client: BaseApiClient::new(base_url, api_key, extra_headers),
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

        let response = self
            .client
            .send_request("chat/completions", &payload)
            .await?;

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

        let stream = self
            .client
            .get_response_stream("chat/completions", &payload)
            .await?;

        Ok(stream)
    }
}
