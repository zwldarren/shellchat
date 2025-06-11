use async_trait::async_trait;
use std::error::Error;

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
    async fn get_response(
        &self,
        messages: &[Message],
        model: &str,
    ) -> Result<String, Box<dyn Error>>;
}

/// Process response text to remove unrelated content
pub fn process_response(content: &str) -> String {
    let trimmed = content.trim();

    // Look for code blocks anywhere in the response
    if let Some(start_pos) = trimmed.find("```") {
        // Find the content after the opening ```
        let after_opening = &trimmed[start_pos + 3..];

        // Skip the language identifier (bash, sh, etc.) if present
        let content_start = if let Some(first_newline) = after_opening.find('\n') {
            first_newline + 1
        } else {
            0
        };

        let code_content = &after_opening[content_start..];

        // Find the closing ``` or take everything if not found
        let code_end = code_content.find("```").unwrap_or(code_content.len());
        let extracted_code = &code_content[..code_end];

        return extracted_code.trim().to_string();
    }

    trimmed.to_string()
}

pub mod openai;
pub mod openrouter;
