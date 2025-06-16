use super::{
    ChatState,
    handler::{
        ClearCommand, DeleteHistoryCommand, DisplayCommand, HelpCommand, ListHistoryCommand,
        LoadHistoryCommand, ModelCommand, QuitCommand, SaveHistoryCommand,
    },
    registry::CommandRegistry,
};
use crate::core::error::SchatError;
use std::sync::Arc;

#[derive(Clone)]
pub struct CommandDispatcher {
    registry: Arc<CommandRegistry>,
}

impl CommandDispatcher {
    pub fn new(registry: Arc<CommandRegistry>) -> Self {
        Self { registry }
    }

    pub fn execute(
        &self,
        command: &str,
        args: &[&str],
        state: &mut ChatState,
    ) -> Result<Option<String>, SchatError> {
        self.registry.execute(command, args, state)
    }

    pub fn get_command_names(&self) -> Vec<String> {
        self.registry.get_command_names()
    }
}

pub fn create_command_registry() -> CommandDispatcher {
    let mut registry = CommandRegistry::new();

    registry.register("quit", QuitCommand);
    registry.register("help", HelpCommand);
    registry.register("clear", ClearCommand);
    registry.register("model", ModelCommand);
    registry.register("save", SaveHistoryCommand);
    registry.register("load", LoadHistoryCommand);
    registry.register("list", ListHistoryCommand);
    registry.register("delete", DeleteHistoryCommand);
    registry.register("display", DisplayCommand);

    CommandDispatcher::new(Arc::new(registry))
}
