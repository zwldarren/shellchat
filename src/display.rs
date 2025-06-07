use console::style;
use std::io::{self};

/// Calculate the display width of a string, accounting for wide characters
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            match c {
            // Most CJK characters take 2 columns
            '\u{1100}'..='\u{115F}' |  // Hangul Jamo
            '\u{2E80}'..='\u{2EFF}' |  // CJK Radicals Supplement
            '\u{2F00}'..='\u{2FDF}' |  // Kangxi Radicals
            '\u{3000}'..='\u{303F}' |  // CJK Symbols and Punctuation
            '\u{3040}'..='\u{309F}' |  // Hiragana
            '\u{30A0}'..='\u{30FF}' |  // Katakana
            '\u{3100}'..='\u{312F}' |  // Bopomofo
            '\u{3130}'..='\u{318F}' |  // Hangul Compatibility Jamo
            '\u{3190}'..='\u{319F}' |  // Kanbun
            '\u{31A0}'..='\u{31BF}' |  // Bopomofo Extended
            '\u{31C0}'..='\u{31EF}' |  // CJK Strokes
            '\u{31F0}'..='\u{31FF}' |  // Katakana Phonetic Extensions
            '\u{3200}'..='\u{32FF}' |  // Enclosed CJK Letters and Months
            '\u{3300}'..='\u{33FF}' |  // CJK Compatibility
            '\u{3400}'..='\u{4DBF}' |  // CJK Unified Ideographs Extension A
            '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
            '\u{A000}'..='\u{A48F}' |  // Yi Syllables
            '\u{A490}'..='\u{A4CF}' |  // Yi Radicals
            '\u{AC00}'..='\u{D7AF}' |  // Hangul Syllables
            '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
            '\u{FE10}'..='\u{FE19}' |  // Vertical Forms
            '\u{FE30}'..='\u{FE4F}' |  // CJK Compatibility Forms
            '\u{FE50}'..='\u{FE6F}' |  // Small Form Variants
            '\u{FF00}'..='\u{FFEF}' |  // Halfwidth and Fullwidth Forms
            '\u{20000}'..='\u{2A6DF}' | // CJK Unified Ideographs Extension B
            '\u{2A700}'..='\u{2B73F}' | // CJK Unified Ideographs Extension C
            '\u{2B740}'..='\u{2B81F}' | // CJK Unified Ideographs Extension D
            '\u{2B820}'..='\u{2CEAF}' | // CJK Unified Ideographs Extension E
            '\u{2CEB0}'..='\u{2EBEF}' => 2, // CJK Unified Ideographs Extension F
            _ => 1,
        }
        })
        .sum()
}

/// Display an AI-generated command in a formatted box
pub fn display_command(command: &str) {
    // Beautiful command display with bash-style formatting - responsive width
    let term = console::Term::stdout();
    let terminal_width = term.size().1 as usize;
    let width = std::cmp::min(terminal_width.saturating_sub(4), 100).max(50);

    // Handle long commands with wrapping or truncation
    let command_lines: Vec<String> = if display_width(command) > width.saturating_sub(6) {
        // Split long commands into multiple lines
        let max_line_len = width.saturating_sub(6); // Account for "│ $ " and "│"
        let mut lines = Vec::new();
        let mut remaining = command;

        while !remaining.is_empty() {
            if display_width(remaining) <= max_line_len {
                lines.push(remaining.to_string());
                break;
            } else {
                // Try to break at a space within display width limit
                let mut break_pos = 0;
                let mut current_width = 0;
                for (pos, ch) in remaining.char_indices() {
                    let char_width = if ch >= '\u{4E00}' && ch <= '\u{9FFF}'
                        || ch >= '\u{3000}' && ch <= '\u{303F}'
                        || ch >= '\u{3040}' && ch <= '\u{309F}'
                        || ch >= '\u{30A0}' && ch <= '\u{30FF}'
                        || ch >= '\u{FF00}' && ch <= '\u{FFEF}'
                    {
                        2
                    } else {
                        1
                    };

                    if current_width + char_width > max_line_len {
                        break;
                    }
                    if ch == ' ' {
                        break_pos = pos;
                    }
                    current_width += char_width;
                }

                if break_pos > 0 {
                    lines.push(remaining[..break_pos].to_string());
                    remaining = remaining[break_pos + 1..].trim_start();
                } else {
                    // No space found, break at display width boundary
                    let mut char_end = 0;
                    let mut current_width = 0;
                    for (pos, ch) in remaining.char_indices() {
                        let char_width = if ch >= '\u{4E00}' && ch <= '\u{9FFF}'
                            || ch >= '\u{3000}' && ch <= '\u{303F}'
                            || ch >= '\u{3040}' && ch <= '\u{309F}'
                            || ch >= '\u{30A0}' && ch <= '\u{30FF}'
                            || ch >= '\u{FF00}' && ch <= '\u{FFEF}'
                        {
                            2
                        } else {
                            1
                        };

                        if current_width + char_width > max_line_len {
                            break;
                        }
                        char_end = pos + ch.len_utf8();
                        current_width += char_width;
                    }
                    lines.push(remaining[..char_end].to_string());
                    remaining = &remaining[char_end..];
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
        let content_len = prompt.len() + display_width(line);
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
        if display_width(line) <= max_width.saturating_sub(4) {
            wrapped_lines.push(line.to_string());
        } else {
            // Wrap long lines
            let max_line_len = max_width.saturating_sub(4); // Account for borders and padding
            let mut remaining = line;

            while !remaining.is_empty() {
                if display_width(remaining) <= max_line_len {
                    wrapped_lines.push(remaining.to_string());
                    break;
                } else {
                    // Find the last space within the display width limit
                    let mut break_pos = 0;
                    let mut current_width = 0;
                    for (pos, ch) in remaining.char_indices() {
                        let char_width = if ch >= '\u{4E00}' && ch <= '\u{9FFF}'
                            || ch >= '\u{3000}' && ch <= '\u{303F}'
                            || ch >= '\u{3040}' && ch <= '\u{309F}'
                            || ch >= '\u{30A0}' && ch <= '\u{30FF}'
                            || ch >= '\u{FF00}' && ch <= '\u{FFEF}'
                        {
                            2
                        } else {
                            1
                        };

                        if current_width + char_width > max_line_len {
                            break;
                        }
                        if ch == ' ' {
                            break_pos = pos;
                        }
                        current_width += char_width;
                    }

                    if break_pos > 0 {
                        wrapped_lines.push(remaining[..break_pos].to_string());
                        remaining = remaining[break_pos + 1..].trim_start();
                    } else {
                        // No space found, break at display width boundary
                        let mut char_end = 0;
                        let mut current_width = 0;
                        for (pos, ch) in remaining.char_indices() {
                            let char_width = if ch >= '\u{4E00}' && ch <= '\u{9FFF}'
                                || ch >= '\u{3000}' && ch <= '\u{303F}'
                                || ch >= '\u{3040}' && ch <= '\u{309F}'
                                || ch >= '\u{30A0}' && ch <= '\u{30FF}'
                                || ch >= '\u{FF00}' && ch <= '\u{FFEF}'
                            {
                                2
                            } else {
                                1
                            };

                            if current_width + char_width > max_line_len {
                                break;
                            }
                            char_end = pos + ch.len_utf8();
                            current_width += char_width;
                        }
                        wrapped_lines.push(remaining[..char_end].to_string());
                        remaining = &remaining[char_end..];
                    }
                }
            }
        }
    }

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
    let bottom_border = "└".to_string() + &"─".repeat(width - 1) + "┘";

    println!("\n{}", style("📤 OUTPUT:").bold().blue());
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
