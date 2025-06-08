use crate::utils::text::{display_width, wrap_text};
use console::style;
use std::env;
use std::io::{self};

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
    let shell_header = format!("┌─ {} ", shell_name)
        + &"─".repeat(width.saturating_sub(9 + shell_name.len()))
        + "┐";
    let shell_footer = "└".to_string() + &"─".repeat(width - 2) + "┘";

    println!("\n{}", style("🤖 AI GENERATED COMMAND").bold().magenta());
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

/// Ask user for execution confirmation
pub fn prompt_execution_confirmation() -> bool {
    let term = console::Term::stdout();
    println!(
        "\n{} {}",
        style("❓").bold().yellow(),
        style("Execute this command? [y/N]").bold().cyan()
    );
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let result = input.trim().eq_ignore_ascii_case("y");
    if result {
        term.clear_last_lines(2).expect("Failed to clear prompt");
    }
    result
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

    println!("\n{}", style("🤖 AI RESPONSE").bold().blue());
    println!("{}", style(&top_border).dim().blue());

    for line in wrapped_lines {
        let padding = box_width.saturating_sub(display_width(&line) + 3);
        println!("│ {}{}│", style(&line).bold().white(), " ".repeat(padding));
    }

    println!("{}", style(&bottom_border).dim().blue());
    println!("{}", style("═".repeat(box_width)).dim());
}

/// Display command stdout output
pub fn display_stdout(output: &[u8]) {
    // Get terminal width and calculate responsive box width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    let top_border = "┌─ stdout ".to_string() + &"─".repeat(width.saturating_sub(10)) + "┐";
    let bottom_border = "└".to_string() + &"─".repeat(width - 1) + "┘";

    println!("{}", style("📤 OUTPUT:").bold().blue());
    println!("{}", style(&top_border).dim().blue());

    // Handle both UTF-8 and non-UTF-8 output
    match String::from_utf8(output.to_vec()) {
        Ok(text) => println!("{}", text),
        Err(e) => {
            // Fall back to lossy UTF-8 conversion for non-UTF-8 output
            let text = String::from_utf8_lossy(e.as_bytes());
            println!("{}", text);
        }
    }

    println!("{}", style(&bottom_border).dim().blue());
}

/// Display command stderr output
pub fn display_stderr(output: &[u8]) {
    // Only show error if there is actual error output
    if !output.is_empty() {
        // Get terminal width and calculate responsive box width
        let term = console::Term::stdout();
        let terminal_width = term.size().1 as usize;
        let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

        let top_border = "┌─ stderr ".to_string() + &"─".repeat(width.saturating_sub(10)) + "┐";
        let bottom_border = "└".to_string() + &"─".repeat(width - 1) + "┘";

        println!("\n{}", style("⚠️  ERROR:").bold().red());
        println!("{}", style(&top_border).dim().red());

        // Handle both UTF-8 and non-UTF-8 output
        match String::from_utf8(output.to_vec()) {
            Ok(text) => eprintln!("{}", text),
            Err(e) => {
                // Fall back to lossy UTF-8 conversion for non-UTF-8 output
                let text = String::from_utf8_lossy(e.as_bytes());
                eprintln!("{}", text);
            }
        }

        println!("{}", style(&bottom_border).dim().red());
    }
}

/// Display command execution status
pub fn display_execution_status(success: bool) {
    // Get terminal width for consistent display
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    // Success indicator
    let status_icon = if success {
        style("✅ COMPLETED").bold().green()
    } else {
        style("❌ FAILED").bold().red()
    };

    println!("{}", status_icon);
    println!("{}", style("═".repeat(width)).dim());
}

/// Display when command execution is cancelled
pub fn display_execution_cancelled() {
    println!(
        "{} {}",
        console::style("🚫").bold().red(),
        console::style("Command execution cancelled").bold().red()
    );
}
