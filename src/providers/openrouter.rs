use super::LLMProvider;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::env;

pub struct OpenRouterProvider;

#[async_trait::async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn get_response(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = env::var("OPENROUTER_API_KEY")?;

        if api_key.trim().is_empty() {
            return Err("OPENROUTER_API_KEY cannot be empty".into());
        }

        // Build the OpenRouter client
        let mut client = OpenAIClient::builder()
            .with_endpoint("https://openrouter.ai/api/v1")
            .with_api_key(api_key)
            .build()?;

        // Create the chat completion request
        let req = ChatCompletionRequest::new(
            model.to_string(),
            vec![
                chat_completion::ChatCompletionMessage {
                    role: chat_completion::MessageRole::system,
                    content: chat_completion::Content::Text(system_prompt.to_string()),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                chat_completion::ChatCompletionMessage {
                    role: chat_completion::MessageRole::user,
                    content: chat_completion::Content::Text(user_prompt.to_string()),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
        );

        // Send the request
        let result = client.chat_completion(req).await?;

        if result.choices.is_empty() {
            return Err("No choices in API response".into());
        }

        let content = match &result.choices[0].message.content {
            Some(text) => text.trim(),
            None => return Err("No content in API response".into()),
        };

        // Process the response to remove code block syntax
        let command = super::process_response(content);

        if command.is_empty() {
            return Err("Empty command received from API".into());
        }

        Ok(command)
    }
}
