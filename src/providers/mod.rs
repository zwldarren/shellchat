use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait ShellCommandProvider {
    async fn get_shell_command(&self, query: &str, model: &str) -> Result<String, Box<dyn Error>>;
}

pub mod openai;
