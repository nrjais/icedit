use crate::text_utils::is_word_boundary;
use ropey::Rope;

/// Convert character column to visual column (accounting for tabs)
fn char_column_to_visual(line: &str, char_column: usize, tab_width: usize) -> usize {
    let mut visual_col = 0;
    let mut char_idx = 0;

    for ch in line.chars() {
        if char_idx >= char_column {
            break;
        }

        if ch == '\t' {
            // Move to next tab stop
            visual_col = ((visual_col / tab_width) + 1) * tab_width;
        } else if ch != '\n' {
            visual_col += 1;
        }
        char_idx += 1;
    }

    visual_col
}

/// Convert visual column to character column (accounting for tabs)
fn visual_column_to_char(line: &str, visual_column: usize, tab_width: usize) -> usize {
    let mut visual_col = 0;
    let mut char_idx = 0;

    for ch in line.chars() {
        if ch == '\t' {
            let next_tab_stop = ((visual_col / tab_width) + 1) * tab_width;
            if visual_column <= visual_col {
                // Visual column is before this tab
                break;
            } else if visual_column < next_tab_stop {
                // Visual column is within this tab's range, position cursor after the tab
                char_idx += 1;
                break;
            } else {
                // Visual column is after this tab, continue
                visual_col = next_tab_stop;
            }
        } else if ch == '\n' {
            break;
        } else {
            if visual_column <= visual_col {
                break;
            }
            visual_col += 1;
        }
        char_idx += 1;
    }

    char_idx
}

/// Represents a position in the text buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn zero() -> Self {
        Self { line: 0, column: 0 }
    }

    /// Convert position to byte offset in the rope
    pub fn to_byte_offset(&self, rope: &Rope) -> usize {
        if self.line >= rope.len_lines() {
            return rope.len_bytes();
        }

        let line_start = rope.line_to_byte(self.line);
        let line = rope.line(self.line);
        // Allow cursor to be anywhere in the line, including after the last character
        // and even at the newline position for selection purposes
        let max_column = line.len_chars();
        let column_chars = std::cmp::min(self.column, max_column);

        // Convert character offset to byte offset within the line
        let line_slice = rope.line(self.line);
        let column_bytes = if column_chars == 0 {
            0
        } else if column_chars >= line_slice.len_chars() {
            line_slice.len_bytes()
        } else {
            line_slice.char_to_byte(column_chars)
        };

        line_start + column_bytes
    }

    /// Create position from byte offset in the rope
    pub fn from_byte_offset(rope: &Rope, offset: usize) -> Self {
        let offset = std::cmp::min(offset, rope.len_bytes());
        let line = rope.byte_to_line(offset);
        let line_start = rope.line_to_byte(line);
        let column = offset - line_start;

        Self { line, column }
    }
}

/// Manages cursor state and movement
#[derive(Debug, Clone, PartialEq)]
pub struct Cursor {
    position: Position,
    desired_visual_column: Option<usize>, // For vertical movement - tracks visual column position
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: Position::zero(),
            desired_visual_column: None,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
        // Reset desired visual column when position is explicitly set
        self.desired_visual_column = None;
    }

    pub fn move_up(&mut self, rope: &Rope) -> bool {
        if self.position.line == 0 {
            return false;
        }

        // Calculate desired visual column if not set
        let desired_visual_col = if let Some(visual_col) = self.desired_visual_column {
            visual_col
        } else {
            // Calculate current visual column from current position
            let current_line = rope.line(self.position.line);
            let current_line_str = current_line.to_string();
            char_column_to_visual(&current_line_str, self.position.column, 4)
        };

        self.position.line -= 1;

        let line = rope.line(self.position.line);
        let line_str = line.to_string();

        // Convert desired visual column back to character column for the new line
        self.position.column = visual_column_to_char(&line_str, desired_visual_col, 4);

        // Ensure we don't go past the end of the line (excluding newline)
        let line_len = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };
        self.position.column = std::cmp::min(self.position.column, line_len);

        // Store the visual column for future vertical movements
        self.desired_visual_column = Some(desired_visual_col);

        true
    }

    pub fn move_down(&mut self, rope: &Rope) -> bool {
        if self.position.line >= rope.len_lines().saturating_sub(1) {
            return false;
        }

        // Calculate desired visual column if not set
        let desired_visual_col = if let Some(visual_col) = self.desired_visual_column {
            visual_col
        } else {
            // Calculate current visual column from current position
            let current_line = rope.line(self.position.line);
            let current_line_str = current_line.to_string();
            char_column_to_visual(&current_line_str, self.position.column, 4)
        };

        self.position.line += 1;

        let line = rope.line(self.position.line);
        let line_str = line.to_string();

        // Convert desired visual column back to character column for the new line
        self.position.column = visual_column_to_char(&line_str, desired_visual_col, 4);

        // Ensure we don't go past the end of the line (excluding newline)
        let line_len = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };
        self.position.column = std::cmp::min(self.position.column, line_len);

        // Store the visual column for future vertical movements
        self.desired_visual_column = Some(desired_visual_col);

        true
    }

    pub fn move_left(&mut self, rope: &Rope) -> bool {
        if self.position.column > 0 {
            self.position.column -= 1;
            // Reset desired visual column for horizontal movement
            self.desired_visual_column = None;
            true
        } else if self.position.line > 0 {
            self.position.line -= 1;
            let line = rope.line(self.position.line);
            // Move to the end of the previous line content, excluding newline
            self.position.column =
                if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
                    line.len_chars() - 1
                } else {
                    line.len_chars()
                };
            // Reset desired visual column for horizontal movement
            self.desired_visual_column = None;
            true
        } else {
            false
        }
    }

    pub fn move_right(&mut self, rope: &Rope) -> bool {
        let line = rope.line(self.position.line);
        // Allow moving to the end of line content, excluding newline for cursor movement
        let line_len = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };

        if self.position.column < line_len {
            self.position.column += 1;
            // Reset desired visual column for horizontal movement
            self.desired_visual_column = None;
            true
        } else if self.position.line < rope.len_lines().saturating_sub(1) {
            self.position.line += 1;
            self.position.column = 0;
            // Reset desired visual column for horizontal movement
            self.desired_visual_column = None;
            true
        } else {
            false
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.position.column = 0;
        // Reset desired visual column
        self.desired_visual_column = None;
    }

    pub fn move_to_line_end(&mut self, rope: &Rope) {
        let line = rope.line(self.position.line);
        // Move to the end of line content, excluding newline
        self.position.column = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };
        // Reset desired visual column
        self.desired_visual_column = None;
    }

    pub fn move_to_document_start(&mut self) {
        self.position = Position::zero();
        // Reset desired visual column
        self.desired_visual_column = None;
    }

    pub fn move_to_document_end(&mut self, rope: &Rope) {
        if rope.len_lines() > 0 {
            self.position.line = rope.len_lines() - 1;
            let line = rope.line(self.position.line);
            // Move to the end of the last line content, excluding newline
            self.position.column =
                if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
                    line.len_chars() - 1
                } else {
                    line.len_chars()
                };
        } else {
            self.position = Position::zero();
        }
        // Reset desired visual column
        self.desired_visual_column = None;
    }

    pub fn move_word_left(&mut self, rope: &Rope) -> bool {
        let current_offset = self.position.to_byte_offset(rope);
        if current_offset == 0 {
            return false;
        }

        let text = rope.slice(..);
        let mut offset = current_offset;

        // Skip boundaries (whitespace and punctuation) backwards until we find a word character
        while offset > 0 {
            let ch = text.char(offset - 1);
            if !is_word_boundary(ch) {
                break;
            }
            offset -= 1;
        }

        // Skip word characters backwards to find the beginning of the word
        while offset > 0 {
            let ch = text.char(offset - 1);
            if is_word_boundary(ch) {
                break;
            }
            offset -= 1;
        }

        self.position = Position::from_byte_offset(rope, offset);
        // Reset desired visual column for word movement
        self.desired_visual_column = None;
        true
    }

    pub fn move_word_right(&mut self, rope: &Rope) -> bool {
        let current_offset = self.position.to_byte_offset(rope);
        if current_offset >= rope.len_bytes() {
            return false;
        }

        let text = rope.slice(..);
        let mut offset = current_offset;

        // Skip current word (non-boundary characters)
        while offset < rope.len_bytes() {
            let ch = text.char(offset);
            if is_word_boundary(ch) {
                break;
            }
            offset += 1;
        }

        // Skip boundaries (whitespace and punctuation) until we find a word character
        while offset < rope.len_bytes() {
            let ch = text.char(offset);
            if !is_word_boundary(ch) {
                break;
            }
            offset += 1;
        }

        self.position = Position::from_byte_offset(rope, offset);
        // Reset desired visual column for word movement
        self.desired_visual_column = None;
        true
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_word_movement_with_special_characters() {
        let rope = Rope::from_str("hello_world.method(param) test-case");
        let mut cursor = Cursor::new();

        // Start at the beginning
        assert_eq!(cursor.position().column, 0);

        // Move right by word - should skip to beginning of next word after "hello_world"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 12); // beginning of "method"

        // Move right by word - should skip to beginning of "param"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 19); // beginning of "param"

        // Move right by word - should skip to beginning of "test-case"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 26); // beginning of "test-case"

        // Now test backward movement
        cursor.move_word_left(&rope);
        assert_eq!(cursor.position().column, 19); // beginning of "param"

        cursor.move_word_left(&rope);
        assert_eq!(cursor.position().column, 12); // beginning of "method"

        cursor.move_word_left(&rope);
        assert_eq!(cursor.position().column, 0); // beginning of "hello_world"
    }

    #[test]
    fn test_word_movement_underscore_hyphen() {
        let rope = Rope::from_str("snake_case kebab-case normal");
        let mut cursor = Cursor::new();

        // Move right by word - should go to beginning of "kebab-case"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 11); // beginning of "kebab-case"

        // Move right by word - should go to beginning of "normal"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 22); // beginning of "normal"
    }

    #[test]
    fn test_word_movement_punctuation() {
        let rope = Rope::from_str("hello,world;test.method(call)");
        let mut cursor = Cursor::new();

        // Test forward movement
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 6); // beginning of "world"

        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 12); // beginning of "test"

        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 17); // beginning of "method"

        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 24); // beginning of "call"
    }

    #[test]
    fn test_word_movement_with_whitespace() {
        let rope = Rope::from_str("  hello   world  ");
        let mut cursor = Cursor::new();

        // Move right - should skip leading whitespace and go to beginning of "hello"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 2); // beginning of "hello"

        // Move right - should go to beginning of "world"
        cursor.move_word_right(&rope);
        assert_eq!(cursor.position().column, 10); // beginning of "world"

        // Move left - should stay at beginning of "world" (already there)
        cursor.move_word_left(&rope);
        assert_eq!(cursor.position().column, 2); // beginning of "hello"
    }

    #[test]
    fn test_position_conversion() {
        let rope = Rope::from_str("line1\nline2_with_underscores\nline3");
        let mut cursor = Cursor::new();

        // Test on first line
        cursor.set_position(Position::new(0, 3));
        assert_eq!(cursor.position().to_byte_offset(&rope), 3);

        // Test on second line
        cursor.set_position(Position::new(1, 5));
        let expected_offset = 6 + 5; // "line1\n" (6 bytes) + 5 chars on second line
        assert_eq!(cursor.position().to_byte_offset(&rope), expected_offset);

        // Test round-trip conversion
        let original_pos = Position::new(1, 10);
        cursor.set_position(original_pos);
        let offset = cursor.position().to_byte_offset(&rope);
        let converted_pos = Position::from_byte_offset(&rope, offset);
        assert_eq!(original_pos.line, converted_pos.line);
        assert_eq!(original_pos.column, converted_pos.column);
    }

    #[test]
    fn test_cursor_movement_with_tabs() {
        let rope = Rope::from_str("hello\tworld\n\tindented\tline\nnormal line");
        let mut cursor = Cursor::new();

        // Test vertical movement with tabs - should maintain visual column position
        // Start at position after "hello" (column 5)
        cursor.set_position(Position::new(0, 5));

        // Move down - should try to maintain visual column 5
        cursor.move_down(&rope);
        // On line "\tindented\tline", visual column 5 would be in the middle of "indented"
        // The tab takes us to visual column 4, then "i" is at visual column 5
        assert_eq!(cursor.position().line, 1);
        assert_eq!(cursor.position().column, 2); // After tab + "i"

        // Move down again to "normal line"
        cursor.move_down(&rope);
        assert_eq!(cursor.position().line, 2);
        assert_eq!(cursor.position().column, 5); // Character 5 in "normal line"

        // Move back up - should maintain the visual column
        cursor.move_up(&rope);
        assert_eq!(cursor.position().line, 1);
        assert_eq!(cursor.position().column, 2); // Back to after tab + "i"

        // Test horizontal movement resets desired visual column
        cursor.move_right(&rope);
        cursor.move_down(&rope); // Should now try to maintain new visual position
        assert_eq!(cursor.position().line, 2);
        // New position should be based on where we were after moving right
    }

    #[test]
    fn test_visual_column_conversion() {
        // Test the utility functions
        let line = "hello\tworld\ttab";

        // Character column 0 = visual column 0
        assert_eq!(char_column_to_visual(line, 0, 4), 0);

        // Character column 5 (just before tab) = visual column 5
        assert_eq!(char_column_to_visual(line, 5, 4), 5);

        // Character column 6 (just after tab) = visual column 8 (next tab stop)
        assert_eq!(char_column_to_visual(line, 6, 4), 8);

        // Test reverse conversion
        assert_eq!(visual_column_to_char(line, 0, 4), 0);
        assert_eq!(visual_column_to_char(line, 5, 4), 5);
        assert_eq!(visual_column_to_char(line, 7, 4), 6); // Visual col 7 maps to after the tab
        assert_eq!(visual_column_to_char(line, 8, 4), 6); // Visual col 8 also maps to after the tab
    }

    #[test]
    fn test_horizontal_movement_with_tabs() {
        let rope = Rope::from_str("a\tb\tc");
        let mut cursor = Cursor::new();

        // Start at beginning: "a\tb\tc"
        //                      ^
        assert_eq!(cursor.position().column, 0);

        // Move right - should go to after 'a' (before tab)
        cursor.move_right(&rope);
        assert_eq!(cursor.position().column, 1); // At the tab character

        // Move right again - should go to after tab (at 'b')
        cursor.move_right(&rope);
        assert_eq!(cursor.position().column, 2); // At 'b'

        // Move right - should go to after 'b' (before second tab)
        cursor.move_right(&rope);
        assert_eq!(cursor.position().column, 3); // At second tab

        // Move right - should go to after second tab (at 'c')
        cursor.move_right(&rope);
        assert_eq!(cursor.position().column, 4); // At 'c'

        // Now test backward movement
        cursor.move_left(&rope);
        assert_eq!(cursor.position().column, 3); // Back to second tab

        cursor.move_left(&rope);
        assert_eq!(cursor.position().column, 2); // Back to 'b'

        cursor.move_left(&rope);
        assert_eq!(cursor.position().column, 1); // Back to first tab

        cursor.move_left(&rope);
        assert_eq!(cursor.position().column, 0); // Back to beginning
    }

    #[test]
    fn test_multiple_consecutive_tabs_movement() {
        let rope = Rope::from_str("\t\tx");
        let mut cursor = Cursor::new();

        // Start at beginning: "\t\tx"
        //                      ^
        assert_eq!(cursor.position().column, 0);

        // Move right - should go to first tab
        cursor.move_right(&rope);
        assert_eq!(cursor.position().column, 1); // At second tab

        // Move right again - should go to after second tab (at 'x')
        cursor.move_right(&rope);
        assert_eq!(cursor.position().column, 2); // At 'x'

        // Test backward movement
        cursor.move_left(&rope);
        assert_eq!(cursor.position().column, 1); // Back to second tab

        cursor.move_left(&rope);
        assert_eq!(cursor.position().column, 0); // Back to first tab
    }
}
