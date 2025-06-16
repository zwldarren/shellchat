use super::ChatState;
use crate::core::error::SchatError;
use crate::providers::Message;

use console::style;

pub trait CommandHandler {
    fn execute(&self, state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError>;
    fn help(&self) -> &'static str;
}

pub struct QuitCommand;
pub struct HelpCommand;
pub struct ClearCommand;
pub struct ModelCommand;
pub struct SaveHistoryCommand;
pub struct LoadHistoryCommand;
pub struct ListHistoryCommand;
pub struct DeleteHistoryCommand;
pub struct DisplayCommand;

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
        let title = style("Available Commands").bold().underlined();
        let help_text = vec![
            title.to_string(),
            style(QuitCommand.help()).to_string(),
            style(HelpCommand.help()).to_string(),
            style(ClearCommand.help()).to_string(),
            style(ModelCommand.help()).to_string(),
            style(SaveHistoryCommand.help()).to_string(),
            style(LoadHistoryCommand.help()).to_string(),
            style(ListHistoryCommand.help()).to_string(),
            style(DeleteHistoryCommand.help()).to_string(),
            style(DisplayCommand.help()).to_string(),
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
            let new_model = args[0].to_string();
            state.provider.set_model(&new_model);
            state.model = new_model;
            Ok(Some(format!("Model changed to: {}", state.model)))
        }
    }

    fn help(&self) -> &'static str {
        "/model <name> - Show or change the current model"
    }
}

impl CommandHandler for SaveHistoryCommand {
    fn execute(&self, state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError> {
        let filename = if args.is_empty() {
            chrono::Local::now()
                .format("%Y%m%d_%H%M%S.json")
                .to_string()
        } else {
            args[0].to_string()
        };

        let history_dir = crate::config::Config::history_dir();
        std::fs::create_dir_all(&history_dir)?;
        let path = history_dir.join(filename);

        let file = std::fs::File::create(&path)?;
        serde_json::to_writer_pretty(file, &state.messages)?;

        Ok(Some(format!("History saved to: {}", path.display())))
    }

    fn help(&self) -> &'static str {
        "/save <filename> - Save conversation history to file"
    }
}

impl CommandHandler for LoadHistoryCommand {
    fn execute(&self, state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError> {
        if args.is_empty() {
            return Ok(Some("Please specify a filename".to_string()));
        }

        let history_dir = crate::config::Config::history_dir();
        let path = history_dir.join(args[0]);

        let file = std::fs::File::open(&path)?;
        state.messages = serde_json::from_reader(file)?;

        // Display loaded messages
        for msg in &state.messages {
            let role = match msg.role {
                crate::providers::Role::System => "System",
                crate::providers::Role::User => "User",
                crate::providers::Role::Assistant => "Assistant",
            };
            println!("\n{}: {}", style(role).bold().cyan(), msg.content);
        }

        Ok(Some(format!("History loaded from: {}", path.display())))
    }

    fn help(&self) -> &'static str {
        "/load <filename> - Load conversation history from file"
    }
}

impl CommandHandler for ListHistoryCommand {
    fn execute(
        &self,
        _state: &mut ChatState,
        _args: &[&str],
    ) -> Result<Option<String>, SchatError> {
        let history_dir = crate::config::Config::history_dir();
        std::fs::create_dir_all(&history_dir)?;

        let mut files = Vec::new();
        for entry in std::fs::read_dir(history_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                files.push(entry.file_name().to_string_lossy().into_owned());
            }
        }

        if files.is_empty() {
            Ok(Some("No history files found.".to_string()))
        } else {
            Ok(Some(format!("{}", files.join("\n"))))
        }
    }

    fn help(&self) -> &'static str {
        "/list - List available conversation history files"
    }
}

impl CommandHandler for DeleteHistoryCommand {
    fn execute(&self, _state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError> {
        if args.is_empty() {
            return Ok(Some("Please specify a filename to delete".to_string()));
        }

        let history_dir = crate::config::Config::history_dir();
        let path = history_dir.join(args[0]);

        if !path.exists() {
            return Ok(Some(format!("File not found: {}", path.display())));
        }

        std::fs::remove_file(&path)?;
        Ok(Some(format!("Deleted history file: {}", path.display())))
    }

    fn help(&self) -> &'static str {
        "/delete <filename> - Delete a conversation history file"
    }
}

impl CommandHandler for DisplayCommand {
    fn execute(&self, _state: &mut ChatState, args: &[&str]) -> Result<Option<String>, SchatError> {
        if args.is_empty() {
            return Ok(Some(
                "Usage: /display <mode> where mode is: verbose, minimal, hidden, or help"
                    .to_string(),
            ));
        }

        match args[0] {
            "verbose" => {
                crate::display::set_display_mode(crate::display::DisplayMode::Verbose);
                Ok(Some(
                    "Display mode set to verbose - showing all tool interactions".to_string(),
                ))
            }
            "minimal" => {
                crate::display::set_display_mode(crate::display::DisplayMode::Minimal);
                Ok(Some(
                    "Display mode set to minimal - showing basic tool activity".to_string(),
                ))
            }
            "hidden" => {
                crate::display::set_display_mode(crate::display::DisplayMode::Hidden);
                Ok(Some(
                    "Display mode set to hidden - hiding all tool interactions".to_string(),
                ))
            }
            "help" => {
                crate::display::display_mode_help();
                Ok(None)
            }
            _ => Ok(Some(
                "Unknown display mode. Use: verbose, minimal, hidden, or help".to_string(),
            )),
        }
    }

    fn help(&self) -> &'static str {
        "/display <mode> - Control tool interaction visibility (verbose/minimal/hidden/help)"
    }
}
