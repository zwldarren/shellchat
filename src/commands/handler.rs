use super::ChatState;
use crate::providers::Message;
use std::error::Error;

pub trait CommandHandler {
    fn execute(
        &self,
        state: &mut ChatState,
        args: &[&str],
    ) -> Result<Option<String>, Box<dyn Error>>;
    fn help(&self) -> &'static str;
}

pub struct QuitCommand;
pub struct HelpCommand;
pub struct ClearCommand;
pub struct ModelCommand;

impl CommandHandler for QuitCommand {
    fn execute(
        &self,
        state: &mut ChatState,
        _args: &[&str],
    ) -> Result<Option<String>, Box<dyn Error>> {
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
    ) -> Result<Option<String>, Box<dyn Error>> {
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
    fn execute(
        &self,
        state: &mut ChatState,
        _args: &[&str],
    ) -> Result<Option<String>, Box<dyn Error>> {
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
    fn execute(
        &self,
        state: &mut ChatState,
        args: &[&str],
    ) -> Result<Option<String>, Box<dyn Error>> {
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
