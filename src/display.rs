use console::{Emoji, Term, style};
use termimad::MadSkin;

/// Represents the user's choice after being prompted for command execution.
#[derive(Debug, PartialEq)]
pub enum UserChoice {
    Execute,
    Describe,
    Abort,
}

/// Display mode for tool interactions
#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    /// Show all tool interactions in detail
    Verbose,
    /// Show minimal tool interaction info
    Minimal,
    /// Hide tool interactions completely
    Hidden,
}

static GEAR: Emoji<'_, '_> = Emoji("⚙️ ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅ ", "");
static CROSS: Emoji<'_, '_> = Emoji("❌ ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "");
static TOOLS: Emoji<'_, '_> = Emoji("🔧 ", "");

/// Global display mode - can be configured
static mut DISPLAY_MODE: DisplayMode = DisplayMode::Minimal;

/// Set the display mode for tool interactions
pub fn set_display_mode(mode: DisplayMode) {
    unsafe {
        DISPLAY_MODE = mode;
    }
}

/// Get current display mode
pub fn get_display_mode() -> DisplayMode {
    unsafe { DISPLAY_MODE }
}

/// Display a beautiful chat header
pub fn display_chat_header() {
    let term = Term::stdout();
    let _ = term.clear_screen();

    let box_width = 63;
    let top_border = "╭".to_string() + &"─".repeat(box_width) + "╮";
    let middle_border = "├".to_string() + &"─".repeat(box_width) + "┤";
    let bottom_border = "╰".to_string() + &"─".repeat(box_width) + "╯";

    println!("{}", style(top_border).dim());

    // Title line - calculate padding to center the text
    let title = "ShellChat - REPL";
    let title_display_len = 16;
    let padding = (box_width - title_display_len) / 2;
    println!(
        "{}{}{}{}{}",
        style("│").dim(),
        " ".repeat(padding),
        style(title).bold().cyan(),
        " ".repeat(box_width - padding - title_display_len),
        style("│").dim()
    );

    println!("{}", style(middle_border).dim());

    // Help line - calculate padding to center the text
    let help_text = "Type your message and press Enter. Use /help for commands.";
    let help_padding = (box_width - help_text.len()) / 2;
    println!(
        "{}{}{}{}{}",
        style("│").dim(),
        " ".repeat(help_padding),
        style(help_text).dim(),
        " ".repeat(box_width - help_padding - help_text.len()),
        style("│").dim()
    );

    println!("{}", style(bottom_border).dim());
    println!();
}

/// Display AI response header
pub fn display_ai_response_header() {
    print!("{} ", style("Assistant:").bold().green());
}

/// Display text with markdown formatting
pub fn display_markdown(text: &str) {
    let skin = MadSkin::default();
    println!();
    skin.print_text(text);
}

/// Display an AI-generated command in plain text
pub fn display_command(command: &str) {
    println!("\n{}", style("COMMAND:").bold().magenta());
    println!("{}", command);
}

/// Ask user for execution confirmation
pub fn prompt_execution_confirmation() -> UserChoice {
    let term = console::Term::stdout();
    print!(
        "\n{}",
        style("[E]xecute, [D]escribe, [A]bort: ").bold().cyan()
    );

    match term.read_line() {
        Ok(input) => {
            let choice = input.trim().to_lowercase();
            if choice == "e" {
                term.clear_last_lines(2).ok(); // Clear prompt and input line
                UserChoice::Execute
            } else if choice == "d" {
                term.clear_last_lines(2).ok(); // Clear prompt and input line
                UserChoice::Describe
            } else {
                // Default to Abort for 'a' or any other input
                UserChoice::Abort
            }
        }
        Err(_) => {
            // If there's an error reading input, default to Abort
            UserChoice::Abort
        }
    }
}

/// Display an AI response in plain text
pub fn display_response(response: &str) {
    println!("\n{}", style("RESPONSE:").bold().blue());
    println!("{}", response);
}

/// Display command stdout output
pub fn display_stdout(output: &[u8]) {
    if output.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(output);
    println!("\n{}", style("OUTPUT:").bold().blue());
    println!("{}", text);
}

/// Display command stderr output
pub fn display_stderr(output: &[u8]) {
    // Only show error if there is actual error output
    if !output.is_empty() {
        let text = String::from_utf8_lossy(output);
        println!("\n{}", style("ERROR:").bold().red());
        println!("{}", text);
    }
}

/// Display tool call information (respects display mode)
pub fn display_tool_call(tool_name: &str) {
    match get_display_mode() {
        DisplayMode::Hidden => return,
        DisplayMode::Minimal => {
            print!("{}{} ", TOOLS, style("using tool...").dim());
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        DisplayMode::Verbose => {
            println!("\n{}{}", GEAR, style("Using tool:").bold().cyan());
            println!("{}", style(tool_name).yellow());
        }
    }
}

/// Display tool arguments (respects display mode)
pub fn display_tool_arguments(args: &str) {
    match get_display_mode() {
        DisplayMode::Hidden | DisplayMode::Minimal => return,
        DisplayMode::Verbose => {
            println!("{}", style("Arguments:").bold().cyan());
            // Pretty print JSON if possible
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(args) {
                if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                    println!("{}", style(pretty).dim());
                    return;
                }
            }
            println!("{}", style(args).dim());
        }
    }
}

/// Display successful tool result (respects display mode)
pub fn display_tool_success(result: &str) {
    match get_display_mode() {
        DisplayMode::Hidden => return,
        DisplayMode::Minimal => {
            let term = Term::stdout();
            // Clear the current line completely
            print!("\r");
            term.clear_line().ok();
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        DisplayMode::Verbose => {
            println!(
                "{}{}",
                CHECK,
                style("Tool completed successfully").bold().green()
            );
            // Try to pretty print JSON results
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result) {
                if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                    println!("{}", style(pretty).dim());
                    return;
                }
            }
            println!("{}", style(result).dim());
        }
    }
}

/// Display tool error (always shown regardless of mode)
pub fn display_tool_error(error: &str) {
    match get_display_mode() {
        DisplayMode::Minimal => {
            // Clear the "正在调用工具..." message
            let term = Term::stdout();
            // Clear the current line completely
            print!("\r");
            term.clear_line().ok();
            println!("{}{}", CROSS, style("Tool failed").red());
        }
        _ => {
            println!("{}{}", CROSS, style("Tool Error:").bold().red());
            println!("{}", style(error).red());
        }
    }
}

/// Display MCP connection message (respects display mode)
pub fn display_mcp_connection(server_name: &str) {
    match get_display_mode() {
        DisplayMode::Hidden => return,
        DisplayMode::Minimal => {
            print!("{}", style(".").dim());
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        DisplayMode::Verbose => {
            println!(
                "{}{} {}",
                GEAR,
                style("Connecting to:").bold().cyan(),
                style(server_name).yellow()
            );
        }
    }
}

/// Display added tool message (respects display mode)
pub fn display_tool_added(tool_name: &str) {
    match get_display_mode() {
        DisplayMode::Hidden => return,
        DisplayMode::Minimal => {
            print!("{}", style(".").dim());
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
        DisplayMode::Verbose => {
            println!(
                "{}{} {}",
                TOOLS,
                style("Added tool:").bold().green(),
                style(tool_name).yellow()
            );
        }
    }
}

/// Display initialization complete message
pub fn display_initialization_complete(tool_count: usize) {
    match get_display_mode() {
        DisplayMode::Hidden => return,
        DisplayMode::Minimal => {
            println!();
            println!(
                "{}{} {} tools ready",
                SPARKLES,
                style("Ready!").bold().green(),
                style(tool_count.to_string()).cyan()
            );
        }
        DisplayMode::Verbose => {
            println!();
            println!(
                "{}{} Initialization complete with {} tools",
                SPARKLES,
                style("Ready!").bold().green(),
                style(tool_count.to_string()).cyan()
            );
        }
    }
    println!();
}

/// Display help for display modes
pub fn display_mode_help() {
    println!("{}", style("Display Modes:").bold().cyan());
    println!(
        "  {} - Show all tool interactions in detail",
        style("verbose").yellow()
    );
    println!(
        "  {} - Show minimal tool activity indicators",
        style("minimal").yellow()
    );
    println!(
        "  {} - Hide all tool interactions",
        style("hidden").yellow()
    );
    println!();
    println!(
        "Current mode: {}",
        match get_display_mode() {
            DisplayMode::Verbose => style("verbose").green(),
            DisplayMode::Minimal => style("minimal").green(),
            DisplayMode::Hidden => style("hidden").green(),
        }
    );
}
