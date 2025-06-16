use crate::commands::handler::CommandHandler;
use crate::core::error::SchatError;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CommandRegistry {
    handlers: HashMap<String, Arc<dyn CommandHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<C: CommandHandler + 'static>(&mut self, name: &str, command: C) {
        self.handlers.insert(name.to_string(), Arc::new(command));
    }

    pub fn execute(
        &self,
        name: &str,
        args: &[&str],
        state: &mut super::ChatState,
    ) -> Result<Option<String>, SchatError> {
        self.handlers
            .get(name)
            .ok_or_else(|| SchatError::Input(format!("Unknown command: {}", name)))
            .and_then(|handler| handler.execute(state, args))
    }

    pub fn get_command_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}
