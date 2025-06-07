use icedit_core::Editor;

/// Utility functions for text measurement and content calculations
/// shared between renderer and widget components.

/// Calculate the width of a line accounting for tabs
///
/// # Arguments
/// * `line` - The line content as a string
/// * `char_width` - Width of a single character
/// * `tab_width` - Width of a tab character (typically 4 * char_width)
///
/// # Returns
/// The total width of the line in pixels
pub fn calculate_line_width(line: &str, char_width: f32, tab_width: f32) -> f32 {
    let mut width = 0.0;

    for ch in line.chars() {
        if ch == '\t' {
            // Tab alignment to next tab stop
            let tab_stop = ((width / tab_width).floor() + 1.0) * tab_width;
            width = tab_stop;
        } else if ch != '\n' {
            width += char_width;
        }
    }
    width
}

/// Calculate the maximum content width for an editor buffer
///
/// # Arguments
/// * `editor` - Reference to the editor instance
/// * `char_width` - Width of a single character
/// * `max_lines_to_check` - Maximum number of lines to check for performance
///
/// # Returns
/// The maximum content width with padding
pub fn calculate_max_content_width(
    editor: &Editor,
    char_width: f32,
    max_lines_to_check: usize,
) -> f32 {
    let rope = editor.current_buffer().rope();
    let line_count = rope.len_lines();
    let mut max_width: f32 = 0.0;
    let tab_width = char_width * 4.0; // 4 spaces per tab

    // Check up to max_lines_to_check lines for performance
    let lines_to_check = line_count.min(max_lines_to_check);

    for line_idx in 0..lines_to_check {
        if let Some(line) = rope.get_line(line_idx) {
            let line_str = line.to_string();
            let width = calculate_line_width(&line_str, char_width, tab_width);
            if width > max_width {
                max_width = width;
            }
        }
    }

    // Add padding to prevent clipping
    max_width + char_width * 2.0
}

/// Calculate character dimensions for a given font size
/// Uses improved calculations based on typical monospace font characteristics
///
/// # Arguments
/// * `font_size` - The font size in pixels
///
/// # Returns
/// A tuple of (char_width, line_height) in pixels
pub fn calculate_char_dimensions(font_size: f32) -> (f32, f32) {
    // For monospace fonts, character width is typically around 0.6 times font size
    let char_width = font_size * 0.6;

    // Line height should be slightly larger than font size for readability
    // 1.2-1.4 is typical, we use 1.3 as a good middle ground
    let line_height = font_size * 1.3;

    // Ensure minimum values to prevent layout issues
    let char_width = char_width.max(1.0);
    let line_height = line_height.max(font_size);

    (char_width, line_height)
}

/// Calculate the X position of a column in a line, accounting for tabs
///
/// # Arguments
/// * `column` - The target column index
/// * `line_content` - The line content as a string
/// * `char_width` - Width of a single character
///
/// # Returns
/// The X position in pixels
pub fn calculate_column_x_position(column: usize, line_content: &str, char_width: f32) -> f32 {
    let mut x = 0.0;
    let mut char_count = 0;
    let tab_width = char_width * 4.0;

    for ch in line_content.chars() {
        if char_count >= column {
            break;
        }

        if ch == '\t' {
            // Tab alignment to next tab stop
            let tab_stop = ((x / tab_width).floor() + 1.0) * tab_width;
            x = tab_stop;
        } else {
            x += char_width;
        }

        char_count += 1;
    }

    x
}

/// Get the default tab width for a given character width
///
/// # Arguments
/// * `char_width` - Width of a single character
///
/// # Returns
/// The tab width (4 characters by default)
pub fn get_tab_width(char_width: f32) -> f32 {
    char_width * 4.0
}

/// Convert a visual X position to a character column index, accounting for tabs
///
/// # Arguments
/// * `x_position` - The target X position in pixels
/// * `line_content` - The line content as a string
/// * `char_width` - Width of a single character
///
/// # Returns
/// The character column index
pub fn x_position_to_column(x_position: f32, line_content: &str, char_width: f32) -> usize {
    let mut current_x = 0.0;
    let mut column = 0;
    let tab_width = get_tab_width(char_width);

    for ch in line_content.chars() {
        let char_width_actual = if ch == '\t' {
            // Calculate tab width to next tab stop
            let tab_stop = ((current_x / tab_width).floor() + 1.0) * tab_width;
            tab_stop - current_x
        } else if ch == '\n' {
            break; // Don't include newline in position calculation
        } else {
            char_width
        };

        // Check if the target position is at or before the middle of this character
        if x_position <= current_x + char_width_actual / 2.0 {
            break;
        }

        current_x += char_width_actual;
        column += 1;
    }

    column
}

/// Calculate the visual width that a column range occupies, accounting for tabs
///
/// # Arguments
/// * `start_column` - Starting column (inclusive)
/// * `end_column` - Ending column (exclusive)
/// * `line_content` - The line content as a string
/// * `char_width` - Width of a single character
///
/// # Returns
/// The visual width in pixels
pub fn calculate_column_range_width(
    start_column: usize,
    end_column: usize,
    line_content: &str,
    char_width: f32,
) -> f32 {
    if start_column >= end_column {
        return 0.0;
    }

    let start_x = calculate_column_x_position(start_column, line_content, char_width);
    let end_x = calculate_column_x_position(end_column, line_content, char_width);
    end_x - start_x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_column_x_position_with_tabs() {
        let char_width = 8.0;
        let line = "a\tb\tc";

        // Position 0 (at 'a') should be at x=0
        assert_eq!(calculate_column_x_position(0, line, char_width), 0.0);

        // Position 1 (at tab) should be at x=8 (after 'a')
        assert_eq!(calculate_column_x_position(1, line, char_width), char_width);

        // Position 2 (at 'b') should be at x=32 (next tab stop after 'a')
        assert_eq!(
            calculate_column_x_position(2, line, char_width),
            char_width * 4.0
        );

        // Position 3 (at second tab) should be at x=40 (after 'b')
        assert_eq!(
            calculate_column_x_position(3, line, char_width),
            char_width * 5.0
        );

        // Position 4 (at 'c') should be at x=64 (next tab stop)
        assert_eq!(
            calculate_column_x_position(4, line, char_width),
            char_width * 8.0
        );
    }

    #[test]
    fn test_x_position_to_column_with_tabs() {
        let char_width = 8.0;
        let line = "a\tb\tc";

        // Let's first understand the layout:
        // 'a' at column 0: x=0 to x=8
        // '\t' at column 1: x=8 to x=32 (tab stop at 32)
        // 'b' at column 2: x=32 to x=40
        // '\t' at column 3: x=40 to x=64 (tab stop at 64)
        // 'c' at column 4: x=64 to x=72

        // X position 0 should map to column 0
        assert_eq!(x_position_to_column(0.0, line, char_width), 0);

        // X position 4 (middle of 'a') should map to column 0
        assert_eq!(x_position_to_column(4.0, line, char_width), 0);

        // X position 8 should map to column 1 (at tab) - actually, this should be column 1 if we're at the exact boundary
        // But let's test what actually happens vs what we expect
        println!(
            "x=8.0 -> column {}",
            x_position_to_column(8.0, line, char_width)
        );

        // X position 16 (middle of tab) should map to column 1 (still in tab)
        println!(
            "x=16.0 -> column {}",
            x_position_to_column(16.0, line, char_width)
        );

        // X position 20 (also in tab) should map to column 1 or 2 depending on where the midpoint is
        println!(
            "x=20.0 -> column {}",
            x_position_to_column(20.0, line, char_width)
        );

        // The tab spans from x=8 to x=32, so its midpoint is at x=20
        // Positions <= 20 should map to column 1 (the tab), positions > 20 should map to column 2
        assert_eq!(x_position_to_column(20.0, line, char_width), 1);

        // X position 32 should map to column 2 (at 'b')
        assert_eq!(x_position_to_column(32.0, line, char_width), 2);
    }

    #[test]
    fn test_multiple_consecutive_tabs() {
        let char_width = 8.0;
        let line = "\t\tx"; // Two tabs followed by 'x'

        // First tab: column 0, x=0 to x=32 (first tab stop)
        // Second tab: column 1, x=32 to x=64 (second tab stop)
        // 'x': column 2, x=64 to x=72

        // Test positions within first tab
        assert_eq!(x_position_to_column(0.0, line, char_width), 0);
        assert_eq!(x_position_to_column(15.0, line, char_width), 0); // Before midpoint at 16
        assert_eq!(x_position_to_column(16.0, line, char_width), 0); // At midpoint
        assert_eq!(x_position_to_column(17.0, line, char_width), 1); // After midpoint

        // Test positions within second tab
        assert_eq!(x_position_to_column(32.0, line, char_width), 1); // Start of second tab
        assert_eq!(x_position_to_column(47.0, line, char_width), 1); // Before midpoint at 48
        assert_eq!(x_position_to_column(48.0, line, char_width), 1); // At midpoint
        assert_eq!(x_position_to_column(49.0, line, char_width), 2); // After midpoint

        // Test position at 'x'
        assert_eq!(x_position_to_column(64.0, line, char_width), 2);
    }

    #[test]
    fn test_calculate_column_x_position_multiple_tabs() {
        let char_width = 8.0;
        let line = "\t\tx";

        // Column 0 (first tab) should be at x=0
        assert_eq!(calculate_column_x_position(0, line, char_width), 0.0);

        // Column 1 (second tab) should be at x=32 (first tab stop)
        assert_eq!(calculate_column_x_position(1, line, char_width), 32.0);

        // Column 2 ('x') should be at x=64 (second tab stop)
        assert_eq!(calculate_column_x_position(2, line, char_width), 64.0);
    }

    #[test]
    fn test_complex_tab_scenarios() {
        let char_width = 8.0;

        // Test line with multiple tabs in different contexts
        let line1 = "hello\t\tworld"; // text + two tabs + text
                                      // 'hello' = 5 chars (x=0 to x=40)
                                      // first tab = x=40 to x=64 (next tab stop at 64)
                                      // second tab = x=64 to x=96 (next tab stop at 96)
                                      // 'world' starts at x=96

        assert_eq!(calculate_column_x_position(5, line1, char_width), 40.0); // First tab position
        assert_eq!(calculate_column_x_position(6, line1, char_width), 64.0); // Second tab position
        assert_eq!(calculate_column_x_position(7, line1, char_width), 96.0); // 'w' in 'world'

        // Test reverse mapping
        assert_eq!(x_position_to_column(40.0, line1, char_width), 5); // Should map to first tab
        assert_eq!(x_position_to_column(64.0, line1, char_width), 6); // Should map to second tab
        assert_eq!(x_position_to_column(96.0, line1, char_width), 7); // Should map to 'w'

        // Test edge case - tabs at beginning followed by various content
        let line2 = "\t\ta\tb";
        // First tab: x=0 to x=32
        // Second tab: x=32 to x=64
        // 'a': x=64 to x=72
        // Third tab: x=72 to x=96
        // 'b': x=96 to x=104

        assert_eq!(calculate_column_x_position(0, line2, char_width), 0.0);
        assert_eq!(calculate_column_x_position(1, line2, char_width), 32.0);
        assert_eq!(calculate_column_x_position(2, line2, char_width), 64.0); // 'a'
        assert_eq!(calculate_column_x_position(3, line2, char_width), 72.0); // third tab
        assert_eq!(calculate_column_x_position(4, line2, char_width), 96.0); // 'b'
    }
}
