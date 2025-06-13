use super::{
    ChatState,
    handler::{ClearCommand, CommandHandler, HelpCommand, ModelCommand, QuitCommand},
};
use crate::error::SchatError;
use std::clone::Clone;
use std::collections::HashMap;

pub struct CommandDispatcher {
    commands: HashMap<&'static str, Box<dyn CommandHandler>>,
}

impl Clone for CommandDispatcher {
    fn clone(&self) -> Self {
        // Create a new instance with the same commands
        let mut new_commands = HashMap::new();

        // Re-register all commands
        new_commands.insert("quit", Box::new(QuitCommand) as Box<dyn CommandHandler>);
        new_commands.insert("help", Box::new(HelpCommand) as Box<dyn CommandHandler>);
        new_commands.insert("clear", Box::new(ClearCommand) as Box<dyn CommandHandler>);
        new_commands.insert("model", Box::new(ModelCommand) as Box<dyn CommandHandler>);

        Self {
            commands: new_commands,
        }
    }
}

impl CommandDispatcher {
    pub fn new() -> Self {
        let mut commands = HashMap::new();

        // Register all commands
        commands.insert("quit", Box::new(QuitCommand) as Box<dyn CommandHandler>);
        commands.insert("help", Box::new(HelpCommand) as Box<dyn CommandHandler>);
        commands.insert("clear", Box::new(ClearCommand) as Box<dyn CommandHandler>);
        commands.insert("model", Box::new(ModelCommand) as Box<dyn CommandHandler>);

        Self { commands }
    }

    /// Returns a list of all registered command names
    pub fn get_command_names(&self) -> Vec<&'static str> {
        self.commands.keys().copied().collect()
    }

    pub fn execute(
        &self,
        command: &str,
        args: &[&str],
        state: &mut ChatState,
    ) -> Result<Option<String>, SchatError> {
        match self.commands.get(command) {
            Some(handler) => handler.execute(state, args),
            None => Ok(Some(format!("Unknown command: {}", command))),
        }
    }
}

pub fn create_command_registry() -> CommandDispatcher {
    CommandDispatcher::new()
}
