use crate::utils::text::{display_width, wrap_text};
use console::style;
use std::env;
use termimad::MadSkin;

/// Display text with markdown formatting
pub fn display_markdown(text: &str) {
    let skin = MadSkin::default();
    println!();
    skin.print_text(text);
}

/// Display an AI-generated command in a formatted box
pub fn display_command(command: &str) {
    // Beautiful command display with bash-style formatting - responsive width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    // Handle long commands with wrapping
    let max_line_len = width.saturating_sub(6); // Account for "│ $ " and "│"
    let command_lines = if display_width(command) > max_line_len {
        wrap_text(command, max_line_len)
    } else {
        vec![command.to_string()]
    };

    let shell_name = if cfg!(target_os = "windows") {
        if env::var("PSModulePath").is_ok() {
            "powershell"
        } else {
            "cmd"
        }
    } else {
        match env::var("SHELL")
            .unwrap_or_else(|_| "/bin/sh".to_string())
            .as_str()
        {
            "/bin/bash" => "bash",
            "/bin/zsh" => "zsh",
            "/bin/fish" => "fish",
            _ => "shell",
        }
    };
    let header_prefix = format!("┌─ {} ", shell_name);
    let repeat_len = width.saturating_sub(display_width(&header_prefix) + 1);
    let shell_header = format!("{}{}{}", header_prefix, "─".repeat(repeat_len), "┐");
    let shell_footer = "└".to_string() + &"─".repeat(width - 2) + "┘";

    println!("\n{}", style("COMMAND:").bold().magenta());
    println!("{}", style(&shell_header).dim().green());

    for (i, line) in command_lines.iter().enumerate() {
        let prompt = if i == 0 { "$ " } else { "  " }; // Only show $ on first line
        let content_len = prompt.len() + display_width(line);
        let padding = width.saturating_sub(content_len + 3); // +3 for borders and spaces

        println!(
            "│ {}{}{}│",
            style(prompt).bold().green(),
            style(line).bold().white(),
            " ".repeat(padding)
        );
    }

    println!("{}", style(&shell_footer).dim().green());
}

/// Represents the user's choice after being prompted for command execution.
#[derive(Debug, PartialEq)]
pub enum UserChoice {
    Execute,
    Describe,
    Abort,
}

/// Ask user for execution confirmation
pub fn prompt_execution_confirmation() -> UserChoice {
    let term = console::Term::stdout();
    println!(
        "\n{}",
        style("Execute this command? [E]xecute, [D]escribe, [A]bort: ")
            .bold()
            .cyan()
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

/// Display an AI response in a formatted box
pub fn display_response(response: &str) {
    // Enhanced AI response display - responsive width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let max_width = std::cmp::min(terminal_width.saturating_sub(4), 120).max(60);

    // Process response lines with text wrapping for long lines
    let max_line_len = max_width.saturating_sub(4); // Account for borders and padding
    let wrapped_lines: Vec<String> = response
        .lines()
        .flat_map(|line| {
            if display_width(line) > max_line_len {
                wrap_text(line, max_line_len)
            } else {
                vec![line.to_string()]
            }
        })
        .collect();

    // Calculate box width based on content and terminal size
    let content_max_len = wrapped_lines
        .iter()
        .map(|line| display_width(line))
        .max()
        .unwrap_or(0);
    let box_width = std::cmp::min(max_width, content_max_len + 4);

    let top_border = "┌".to_string() + &"─".repeat(box_width - 2) + "┐";
    let bottom_border = "└".to_string() + &"─".repeat(box_width - 2) + "┘";

    println!("\n{}", style("RESPONSE: ").bold().blue());
    println!("{}", style(&top_border).dim().blue());

    for line in wrapped_lines {
        let padding = box_width.saturating_sub(display_width(&line) + 3);
        println!("│ {}{}│", style(&line).bold().white(), " ".repeat(padding));
    }

    println!("{}", style(&bottom_border).dim().blue());
}

/// Display command stdout output
pub fn display_stdout(output: &[u8]) {
    if output.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(output);

    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let max_width = std::cmp::min(terminal_width.saturating_sub(4), 120).max(60);

    let max_line_len = max_width.saturating_sub(4);
    let wrapped_lines: Vec<String> = text
        .lines()
        .flat_map(|line| {
            if display_width(line) > max_line_len {
                wrap_text(line, max_line_len)
            } else {
                vec![line.to_string()]
            }
        })
        .collect();

    let content_max_len = wrapped_lines
        .iter()
        .map(|line| display_width(line))
        .max()
        .unwrap_or(0);

    let header_prefix = "┌─ stdout ";
    let header_len = display_width(header_prefix) + 1;
    let box_width = std::cmp::min(max_width, std::cmp::max(content_max_len + 4, header_len));

    let repeat_len = box_width.saturating_sub(display_width(header_prefix) + 1);
    let top_border = format!("{}{}{}", header_prefix, "─".repeat(repeat_len), "┐");
    let bottom_border = "└".to_string() + &"─".repeat(box_width - 2) + "┘";

    println!("\n{}", style("OUTPUT:").bold().blue());
    println!("{}", style(&top_border).dim().blue());

    for line in wrapped_lines {
        let padding = box_width.saturating_sub(display_width(&line) + 3);
        println!("│ {}{}│", style(&line).bold().white(), " ".repeat(padding));
    }

    println!("{}", style(&bottom_border).dim().blue());
}

/// Display command stderr output
pub fn display_stderr(output: &[u8]) {
    // Only show error if there is actual error output
    if !output.is_empty() {
        let text = String::from_utf8_lossy(output);

        let term = console::Term::stdout();
        let terminal_width = term.size().1 as usize;
        let max_width = std::cmp::min(terminal_width.saturating_sub(4), 120).max(60);

        let max_line_len = max_width.saturating_sub(4);
        let wrapped_lines: Vec<String> = text
            .lines()
            .flat_map(|line| {
                if display_width(line) > max_line_len {
                    wrap_text(line, max_line_len)
                } else {
                    vec![line.to_string()]
                }
            })
            .collect();

        let content_max_len = wrapped_lines
            .iter()
            .map(|line| display_width(line))
            .max()
            .unwrap_or(0);

        let header_prefix = "┌─ stderr ";
        let header_len = display_width(header_prefix) + 1;
        let box_width = std::cmp::min(max_width, std::cmp::max(content_max_len + 4, header_len));

        let repeat_len = box_width.saturating_sub(display_width(header_prefix) + 1);
        let top_border = format!("{}{}{}", header_prefix, "─".repeat(repeat_len), "┐");
        let bottom_border = "└".to_string() + &"─".repeat(box_width - 2) + "┘";

        println!("\n{}", style("ERROR:").bold().red());
        println!("{}", style(&top_border).dim().red());

        for line in wrapped_lines {
            let padding = box_width.saturating_sub(display_width(&line) + 3);
            println!("│ {}{}│", style(&line).bold().red(), " ".repeat(padding));
        }

        println!("{}", style(&bottom_border).dim().red());
    }
}
