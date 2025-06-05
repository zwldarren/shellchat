use console::style;
use std::io::{self, Write};

/// Display an AI-generated command in a formatted box
pub fn display_command(command: &str) {
    // Beautiful command display with bash-style formatting - responsive width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    // Handle long commands with wrapping or truncation
    let command_lines: Vec<String> = if command.len() > width.saturating_sub(6) {
        // Split long commands into multiple lines
        let max_line_len = width.saturating_sub(6); // Account for "│ $ " and "│"
        let mut lines = Vec::new();
        let mut remaining = command.to_string();

        while !remaining.is_empty() {
            if remaining.len() <= max_line_len {
                lines.push(remaining.to_string());
                break;
            } else {
                // Try to break at a space near the limit
                if let Some(break_pos) = remaining[..max_line_len].rfind(' ') {
                    lines.push(remaining[..break_pos].to_string());
                    remaining = remaining[break_pos + 1..].trim_start().to_string();
                } else {
                    // No good break point, just cut at the limit
                    lines.push(remaining[..max_line_len].to_string());
                    remaining = remaining[max_line_len..].to_string();
                }
            }
        }
        lines
    } else {
        vec![command.to_string()]
    };

    let bash_header = "┌─ bash ".to_string() + &"─".repeat(width.saturating_sub(9)) + "┐";
    let bash_footer = "└".to_string() + &"─".repeat(width - 2) + "┘";

    println!("\n{}", style("🤖 AI GENERATED COMMAND").bold().magenta());
    println!("{}", style(&bash_header).dim().green());

    for (i, line) in command_lines.iter().enumerate() {
        let prompt = if i == 0 { "$ " } else { "  " }; // Only show $ on first line
        let content_len = prompt.len() + line.len();
        let padding = width.saturating_sub(content_len + 3); // +3 for borders and spaces

        println!(
            "│ {}{}{}│",
            style(prompt).bold().green(),
            style(line).bold().white(),
            " ".repeat(padding)
        );
    }

    println!("{}", style(&bash_footer).dim().green());
}

/// Ask user for execution confirmation
pub fn prompt_execution_confirmation() -> bool {
    println!(
        "\n{} {}",
        style("❓").bold().yellow(),
        style("Execute this command? [y/N]").bold().cyan()
    );
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().eq_ignore_ascii_case("y")
}

/// Display an AI response in a formatted box
pub fn display_response(response: &str) {
    // Enhanced AI response display - responsive width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let max_width = std::cmp::min(terminal_width.saturating_sub(4), 120).max(60);

    // Process response lines with text wrapping for long lines
    let mut wrapped_lines = Vec::new();
    for line in response.lines() {
        if line.len() <= max_width.saturating_sub(4) {
            wrapped_lines.push(line.to_string());
        } else {
            // Wrap long lines
            let max_line_len = max_width.saturating_sub(4); // Account for borders and padding
            let mut remaining = line;

            while !remaining.is_empty() {
                if remaining.len() <= max_line_len {
                    wrapped_lines.push(remaining.to_string());
                    break;
                } else {
                    // Try to break at a space near the limit
                    if let Some(break_pos) = remaining[..max_line_len].rfind(' ') {
                        wrapped_lines.push(remaining[..break_pos].to_string());
                        remaining = remaining[break_pos + 1..].trim_start();
                    } else {
                        // No good break point, just cut at the limit
                        wrapped_lines.push(remaining[..max_line_len].to_string());
                        remaining = &remaining[max_line_len..];
                    }
                }
            }
        }
    }

    // Calculate box width based on content and terminal size
    let content_max_len = wrapped_lines
        .iter()
        .map(|line| line.len())
        .max()
        .unwrap_or(0);
    let box_width = std::cmp::min(max_width, content_max_len + 4);

    let top_border = "┌".to_string() + &"─".repeat(box_width - 2) + "┐";
    let bottom_border = "└".to_string() + &"─".repeat(box_width - 2) + "┘";

    println!("\n{}", style("🤖 AI RESPONSE").bold().blue());
    println!("{}", style(&top_border).dim().blue());

    for line in wrapped_lines {
        let padding = box_width.saturating_sub(line.len() + 3);
        println!("│ {}{}│", style(&line).bold().white(), " ".repeat(padding));
    }

    println!("{}", style(&bottom_border).dim().blue());
    println!("{}", style("═".repeat(box_width)).dim());
}

/// Display command execution banner
pub fn display_execution_banner(command: &str) {
    // Get terminal width and calculate responsive box width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    // Wrap command text if it's too long
    let command_display = if command.len() > width.saturating_sub(16) {
        let max_cmd_len = width.saturating_sub(19); // Account for "⚡ EXECUTING: " + borders + "..."
        format!("{}...", &command[..max_cmd_len])
    } else {
        command.to_string()
    };

    let top_border = "┌".to_string() + &"─".repeat(width - 2) + "┐";
    let bottom_border = "└".to_string() + &"─".repeat(width - 2) + "┘";

    println!("\n{}", style(&top_border).dim().cyan());

    let label = "⚡ EXECUTING:";
    let content_len = label.len() + command_display.len() + 1; // +1 for space
    let padding = width.saturating_sub(content_len + 3); // +3 for borders and spaces

    println!(
        "│ {} {}{} │",
        style(label).bold().green(),
        style(&command_display).bold().yellow(),
        " ".repeat(padding)
    );
    println!("{}", style(&bottom_border).dim().cyan());

    // Add a subtle separator
    println!("{}", style("▼".repeat(width)).dim().blue());
}

/// Display command stdout output
pub fn display_stdout(output: &[u8]) {
    // Get terminal width and calculate responsive box width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    let top_border = "┌─ stdout ".to_string() + &"─".repeat(width.saturating_sub(10)) + "┐";
    let bottom_border = "└".to_string() + &"─".repeat(width - 2) + "┘";

    println!("\n{}", style("📤 OUTPUT:").bold().blue());
    println!("{}", style(&top_border).dim().blue());
    io::stdout()
        .write_all(output)
        .expect("Failed to write stdout");
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
        io::stderr()
            .write_all(output)
            .expect("Failed to write stderr");
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

    println!("\n{}", style("▲".repeat(width)).dim().blue());
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
