use crate::error::SchatError;
use async_trait::async_trait;
use futures::stream::BoxStream;

#[derive(Debug, Clone, Copy)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[async_trait]
pub trait LLMProvider {
    async fn get_response(&self, messages: &[Message], model: &str) -> Result<String, SchatError>;

    async fn get_response_stream(
        &self,
        messages: &[Message],
        model: &str,
    ) -> Result<BoxStream<'static, Result<String, SchatError>>, SchatError>;
}

/// Process response text to extract command or code block
pub fn process_response(content: &str) -> String {
    let content = content.trim();

    // Handle empty response
    if content.is_empty() {
        return String::new();
    }

    // Look for code blocks
    if let Some(start_idx) = content.find("```") {
        let after_start = &content[start_idx + 3..];
        let end_idx = after_start.find("```").map(|i| i + start_idx + 3);

        let code_block = if let Some(end_idx) = end_idx {
            &content[start_idx + 3..end_idx]
        } else {
            &content[start_idx + 3..]
        };

        // Remove language specifier if present
        if let Some(first_newline) = code_block.find('\n') {
            return code_block[first_newline + 1..].trim().to_string();
        }
        return code_block.trim().to_string();
    }

    // Look for command in quotes
    if let Some(start) = content.find('`') {
        if let Some(end) = content[start + 1..].find('`').map(|i| i + start + 1) {
            return content[start + 1..end].trim().to_string();
        }
    }

    // Fallback: return first non-empty line
    content
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| content.to_string())
}

pub mod base_client;
pub mod openai;
pub mod openrouter;
