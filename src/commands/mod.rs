pub mod dispatcher;
pub mod handler;
pub mod registry;

use crate::providers::{LLMProvider, Message};
pub use dispatcher::create_command_registry;

pub struct ChatState {
    pub messages: Vec<Message>,
    pub provider: Box<dyn LLMProvider>,
    pub model: String,
    pub should_continue: bool,
}

impl ChatState {
    pub fn new(provider: Box<dyn LLMProvider>, model: &str) -> Self {
        Self {
            messages: vec![Message {
                role: crate::providers::Role::System,
                content: "You are a helpful assistant.".to_string(),
            }],
            provider,
            model: model.to_string(),
            should_continue: true,
        }
    }
}
