use super::LLMProvider;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatCompletionMessage<'a>>,
}

#[derive(Serialize)]
struct ChatCompletionMessage<'a> {
    role: &'a str,
    content: &'a str,
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
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

impl OpenAIProvider {
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
impl LLMProvider for OpenAIProvider {
    async fn get_response(
        &self,
        messages: &[super::Message],
        model: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => env::var("OPENAI_API_KEY")
                .map_err(|_| "OPENAI_API_KEY must be set from config or environment variable")?,
        };

        if api_key.trim().is_empty() {
            return Err("OPENAI_API_KEY cannot be empty".into());
        }

        let client = Client::builder().build()?;

        let req_messages: Vec<ChatCompletionMessage> = messages
            .iter()
            .map(|m| ChatCompletionMessage {
                role: match m.role {
                    super::Role::System => "system",
                    super::Role::User => "user",
                    super::Role::Assistant => "assistant",
                },
                content: &m.content,
            })
            .collect();

        let payload = ChatCompletionRequest {
            model,
            messages: req_messages,
        };

        let endpoint = self
            .endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let url = format!("{}/chat/completions", endpoint);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json::<ChatCompletionResponse>()
            .await?;

        if response.choices.is_empty() {
            return Err("No choices in API response".into());
        }

        let content = response.choices[0].message.content.trim().to_string();

        if content.is_empty() {
            return Err("Empty command received from API".into());
        }

        Ok(content)
    }
}
