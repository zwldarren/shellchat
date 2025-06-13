use console::style;
use termimad::MadSkin;

/// Represents the user's choice after being prompted for command execution.
#[derive(Debug, PartialEq)]
pub enum UserChoice {
    Execute,
    Describe,
    Abort,
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
