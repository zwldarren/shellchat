use super::ChatState;
use crate::error::SchatError;
use crate::providers::Message;

pub trait CommandHandler {
    fn execute(&self, state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError>;
    fn help(&self) -> &'static str;
}

pub struct QuitCommand;
pub struct HelpCommand;
pub struct ClearCommand;
pub struct ModelCommand;

impl CommandHandler for QuitCommand {
    fn execute(&self, state: &mut ChatState, _args: &[&str]) -> Result<Option<String>, SchatError> {
        state.should_continue = false;
        Ok(None)
    }

    fn help(&self) -> &'static str {
        "/quit - Exit the chat session"
    }
}

impl CommandHandler for HelpCommand {
    fn execute(
        &self,
        _state: &mut ChatState,
        _args: &[&str],
    ) -> Result<Option<String>, SchatError> {
        let help_text = vec![
            "Available commands:",
            QuitCommand.help(),
            HelpCommand.help(),
            ClearCommand.help(),
            ModelCommand.help(),
        ]
        .join("\n");

        Ok(Some(help_text))
    }

    fn help(&self) -> &'static str {
        "/help - Show available commands"
    }
}

impl CommandHandler for ClearCommand {
    fn execute(&self, state: &mut ChatState, _args: &[&str]) -> Result<Option<String>, SchatError> {
        state.messages = vec![Message {
            role: crate::providers::Role::System,
            content: "You are a helpful assistant.".to_string(),
        }];
        Ok(Some("Chat history cleared.".to_string()))
    }

    fn help(&self) -> &'static str {
        "/clear - Clear conversation history"
    }
}

impl CommandHandler for ModelCommand {
    fn execute(&self, state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError> {
        if args.is_empty() {
            Ok(Some(format!("Current model: {}", state.model)))
        } else {
            state.model = args[0].to_string();
            Ok(Some(format!("Model changed to: {}", state.model)))
        }
    }

    fn help(&self) -> &'static str {
        "/model [name] - Show or change the current model"
    }
}
