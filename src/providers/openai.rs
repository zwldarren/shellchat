use super::LLMProvider;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

pub struct OpenAIProvider;

#[async_trait::async_trait]
impl LLMProvider for OpenAIProvider {
    async fn get_shell_command(
        &self,
        query: &str,
        model: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        if api_key.trim().is_empty() {
            return Err("OPENAI_API_KEY cannot be empty".into());
        }

        let request = OpenAIRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: format!("Convert this to a single bash command: {}", query),
            }],
            temperature: 0.0,
        };

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API request failed: {}", error_text).into());
        }

        let api_response: OpenAIResponse = response.json().await?;

        if api_response.choices.is_empty() {
            return Err("No choices in API response".into());
        }

        let content = api_response.choices[0].message.content.trim();

        let command = if content.starts_with("```") {
            content
                .trim_start_matches("```bash")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
                .to_string()
        } else {
            content.lines().next().unwrap_or("").trim().to_string()
        };

        if command.is_empty() {
            return Err("Empty command received from API".into());
        }

        Ok(command)
    }
}
