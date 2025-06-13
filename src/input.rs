use crate::commands::dispatcher::CommandDispatcher;
use crate::error::SchatError;

use console::style;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::history::FileHistory;
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use std::borrow::Cow;
use std::path::Path;

/// Custom completer that combines filename and command completion
pub struct ShellCompleter {
    filename_completer: FilenameCompleter,
    command_registry: CommandDispatcher,
}

impl ShellCompleter {
    pub fn new(command_registry: CommandDispatcher) -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            command_registry,
        }
    }
}

impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // If the line starts with '/', it's a command
        if line.starts_with('/') {
            let command_part = &line[1..pos];

            // Get all command names
            let commands = self.command_registry.get_command_names();

            // Filter commands that match the current input
            let matches: Vec<Pair> = commands
                .iter()
                .filter(|cmd| cmd.starts_with(command_part))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();

            if !matches.is_empty() {
                return Ok((1, matches)); // 1 is the position after '/'
            }
        }

        // Default to filename completion
        self.filename_completer.complete(line, pos, ctx)
    }
}

/// Custom highlighter for syntax highlighting
pub struct ShellHighlighter {
    bracket_highlighter: MatchingBracketHighlighter,
}

impl ShellHighlighter {
    pub fn new() -> Self {
        Self {
            bracket_highlighter: MatchingBracketHighlighter::new(),
        }
    }
}

impl Highlighter for ShellHighlighter {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.bracket_highlighter.highlight(line, pos)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        self.bracket_highlighter
            .highlight_candidate(candidate, completion)
    }
}

/// Custom hinter that provides suggestions
pub struct ShellHinter {
    history_hinter: HistoryHinter,
}

impl ShellHinter {
    pub fn new() -> Self {
        Self {
            history_hinter: HistoryHinter {},
        }
    }
}

impl Hinter for ShellHinter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        // If line starts with '/', provide command hints
        if line.starts_with('/') && pos > 1 {
            // Command hints would go here
            // For now, just use history hints
            return self.history_hinter.hint(line, pos, ctx);
        }

        // Otherwise use history-based hints
        self.history_hinter.hint(line, pos, ctx)
    }
}

/// Custom validator for input validation
pub struct ShellValidator {
    bracket_validator: MatchingBracketValidator,
}

impl ShellValidator {
    pub fn new() -> Self {
        Self {
            bracket_validator: MatchingBracketValidator::new(),
        }
    }
}

impl Validator for ShellValidator {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.bracket_validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.bracket_validator.validate_while_typing()
    }
}

/// Helper struct that combines all rustyline components
pub struct ShellHelper {
    completer: ShellCompleter,
    highlighter: ShellHighlighter,
    hinter: ShellHinter,
    validator: ShellValidator,
}

impl ShellHelper {
    pub fn new(command_registry: CommandDispatcher) -> Self {
        Self {
            completer: ShellCompleter::new(command_registry),
            highlighter: ShellHighlighter::new(),
            hinter: ShellHinter::new(),
            validator: ShellValidator::new(),
        }
    }
}

impl Helper for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ShellHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        self.highlighter.highlight_hint(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        self.highlighter.highlight_candidate(candidate, completion)
    }
}

impl Validator for ShellHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

/// Creates a configured rustyline editor
pub fn create_editor(
    command_registry: CommandDispatcher,
) -> Result<Editor<ShellHelper, FileHistory>, SchatError> {
    // Configure rustyline with all the features we want
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    let mut editor = Editor::with_config(config)
        .map_err(|e| SchatError::Input(format!("Failed to create line editor: {}", e)))?;

    let helper = ShellHelper::new(command_registry);
    editor.set_helper(Some(helper));

    let history_path = dirs::home_dir()
        .map(|mut path| {
            path.push(".schat/input_history.txt");
            path
        })
        .unwrap_or_else(|| Path::new(".schat/input_history.txt").to_path_buf());
    let _ = editor.load_history(&history_path);

    Ok(editor)
}

/// Reads a line of input using rustyline
pub fn read_input(
    editor: &mut Editor<ShellHelper, FileHistory>,
) -> Result<Option<String>, SchatError> {
    let prompt = if cfg!(windows) && std::env::var("PSModulePath").is_ok() {
        "> ".to_string()
    } else {
        style("> ").bold().cyan().to_string()
    };
    match editor.readline(&prompt) {
        Ok(line) => {
            // Add to history if non-empty and starts with '/'
            if !line.trim().is_empty() && line.starts_with('/') {
                if let Err(e) = editor.add_history_entry(&line) {
                    return Err(SchatError::Input(format!(
                        "Failed to add history entry: {}",
                        e
                    )));
                }
            }
            Ok(Some(line))
        }
        Err(ReadlineError::Interrupted) => {
            // Ctrl-C pressed
            println!("Exiting...");
            Ok(None)
        }
        Err(ReadlineError::Eof) => {
            // Ctrl-D pressed
            println!("Exiting...");
            Ok(None)
        }
        Err(err) => Err(SchatError::Input(format!("Input error: {}", err))),
    }
}

/// Saves the editor history
pub fn save_history(editor: &mut Editor<ShellHelper, FileHistory>) -> Result<(), SchatError> {
    let history_path = dirs::home_dir()
        .map(|mut path| {
            path.push(".schat/input_history.txt");
            path
        })
        .unwrap_or_else(|| Path::new(".schat/input_history.txt").to_path_buf());

    // Ensure the history directory exists
    if let Some(parent) = history_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SchatError::Input(format!("Failed to create history directory: {}", e))
            })?;
        }
    }

    editor
        .save_history(&history_path)
        .map_err(|e| SchatError::Input(format!("Failed to save history: {}", e)))
}
