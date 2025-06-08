/// Calculate the display width of a string, accounting for wide characters
pub fn display_width(s: &str) -> usize {
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

/// Wrap a string into lines with a given maximum display width.
pub fn wrap_text(text: &str, max_line_len: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if display_width(remaining) <= max_line_len {
            lines.push(remaining.to_string());
            break;
        } else {
            // Try to break at a space within display width limit
            let mut break_pos = 0;
            let mut current_width = 0;
            for (pos, ch) in remaining.char_indices() {
                let char_width = display_width(&ch.to_string());

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
                    let char_width = display_width(&ch.to_string());

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
}
