use ropey::Rope;

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
    desired_column: Option<usize>, // For vertical movement
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: Position::zero(),
            desired_column: None,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
        self.desired_column = Some(position.column);
    }

    pub fn move_up(&mut self, rope: &Rope) -> bool {
        if self.position.line == 0 {
            return false;
        }

        let desired_col = self.desired_column.unwrap_or(self.position.column);
        self.position.line -= 1;

        let line = rope.line(self.position.line);
        // Allow cursor to be at the end of line content, excluding only the newline for movement
        let line_len = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };
        self.position.column = std::cmp::min(desired_col, line_len);

        true
    }

    pub fn move_down(&mut self, rope: &Rope) -> bool {
        if self.position.line >= rope.len_lines().saturating_sub(1) {
            return false;
        }

        let desired_col = self.desired_column.unwrap_or(self.position.column);
        self.position.line += 1;

        let line = rope.line(self.position.line);
        // Allow cursor to be at the end of line content, excluding only the newline for movement
        let line_len = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };
        self.position.column = std::cmp::min(desired_col, line_len);

        true
    }

    pub fn move_left(&mut self, rope: &Rope) -> bool {
        if self.position.column > 0 {
            self.position.column -= 1;
            self.desired_column = Some(self.position.column);
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
            self.desired_column = Some(self.position.column);
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
            self.desired_column = Some(self.position.column);
            true
        } else if self.position.line < rope.len_lines().saturating_sub(1) {
            self.position.line += 1;
            self.position.column = 0;
            self.desired_column = Some(0);
            true
        } else {
            false
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.position.column = 0;
        self.desired_column = Some(0);
    }

    pub fn move_to_line_end(&mut self, rope: &Rope) {
        let line = rope.line(self.position.line);
        // Move to the end of line content, excluding newline
        self.position.column = if line.len_chars() > 0 && line.char(line.len_chars() - 1) == '\n' {
            line.len_chars() - 1
        } else {
            line.len_chars()
        };
        self.desired_column = Some(self.position.column);
    }

    pub fn move_to_document_start(&mut self) {
        self.position = Position::zero();
        self.desired_column = Some(0);
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
        self.desired_column = Some(self.position.column);
    }

    pub fn move_word_left(&mut self, rope: &Rope) -> bool {
        let current_offset = self.position.to_byte_offset(rope);
        if current_offset == 0 {
            return false;
        }

        let text = rope.slice(..);
        let mut offset = current_offset;

        // Skip whitespace backwards
        while offset > 0 {
            let ch = text.char(offset - 1);
            if !ch.is_whitespace() {
                break;
            }
            offset -= 1;
        }

        // Skip word characters backwards
        while offset > 0 {
            let ch = text.char(offset - 1);
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                break;
            }
            offset -= 1;
        }

        self.position = Position::from_byte_offset(rope, offset);
        self.desired_column = Some(self.position.column);
        true
    }

    pub fn move_word_right(&mut self, rope: &Rope) -> bool {
        let current_offset = self.position.to_byte_offset(rope);
        if current_offset >= rope.len_bytes() {
            return false;
        }

        let text = rope.slice(..);
        let mut offset = current_offset;

        // Skip current word
        while offset < rope.len_bytes() {
            let ch = text.char(offset);
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                break;
            }
            offset += 1;
        }

        // Skip whitespace
        while offset < rope.len_bytes() {
            let ch = text.char(offset);
            if !ch.is_whitespace() {
                break;
            }
            offset += 1;
        }

        self.position = Position::from_byte_offset(rope, offset);
        self.desired_column = Some(self.position.column);
        true
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
