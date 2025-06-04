use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait LLMProvider {
    async fn get_response(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        model: &str,
    ) -> Result<String, Box<dyn Error>>;
}

/// Process response text to remove markdown code block syntax
pub fn process_response(content: &str) -> String {
    let trimmed = content.trim();

    // Check if content is wrapped in code blocks
    if trimmed.starts_with("```") {
        // Find the first newline after the opening ```
        if let Some(first_newline) = trimmed.find('\n') {
            let mut result = &trimmed[first_newline + 1..];

            // Remove trailing ``` if present
            if result.ends_with("```") {
                result = &result[..result.len() - 3];
            }

            return result.trim().to_string();
        }
    }

    trimmed.to_string()
}

pub mod openai;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_response_with_bash_code_block() {
        let input = "```bash\nls -la\n```";
        let expected = "ls -la";
        assert_eq!(process_response(input), expected);
    }

    #[test]
    fn test_process_response_with_sh_code_block() {
        let input = "```sh\necho hello\n```";
        let expected = "echo hello";
        assert_eq!(process_response(input), expected);
    }

    #[test]
    fn test_process_response_with_shell_code_block() {
        let input = "```shell\ncd /home\npwd\n```";
        let expected = "cd /home\npwd";
        assert_eq!(process_response(input), expected);
    }

    #[test]
    fn test_process_response_without_code_block() {
        let input = "ls -la";
        let expected = "ls -la";
        assert_eq!(process_response(input), expected);
    }

    #[test]
    fn test_process_response_with_whitespace() {
        let input = "  ```bash\n  ls -la  \n```  ";
        let expected = "ls -la";
        assert_eq!(process_response(input), expected);
    }

    #[test]
    fn test_process_response_empty_code_block() {
        let input = "```bash\n```";
        let expected = "";
        assert_eq!(process_response(input), expected);
    }
}
