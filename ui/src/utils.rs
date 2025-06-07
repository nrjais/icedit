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
