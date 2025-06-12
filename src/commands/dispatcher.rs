use super::{
    ChatState,
    handler::{ClearCommand, CommandHandler, HelpCommand, ModelCommand, QuitCommand},
};
use std::{collections::HashMap, error::Error};

pub struct CommandDispatcher {
    commands: HashMap<&'static str, Box<dyn CommandHandler>>,
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

    pub fn execute(
        &self,
        command: &str,
        args: &[&str],
        state: &mut ChatState,
    ) -> Result<Option<String>, Box<dyn Error>> {
        match self.commands.get(command) {
            Some(handler) => handler.execute(state, args),
            None => Ok(Some(format!("Unknown command: {}", command))),
        }
    }
}

pub fn create_command_registry() -> CommandDispatcher {
    CommandDispatcher::new()
}
