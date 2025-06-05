use crate::Position;
use ropey::Rope;

/// Represents a text selection range
#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
}

impl Selection {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a selection from two positions, ensuring start <= end
    pub fn from_positions(pos1: Position, pos2: Position) -> Self {
        if pos1.line < pos2.line || (pos1.line == pos2.line && pos1.column <= pos2.column) {
            Self::new(pos1, pos2)
        } else {
            Self::new(pos2, pos1)
        }
    }

    /// Check if the selection is empty (start == end)
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the selected text from the rope
    pub fn get_text(&self, rope: &Rope) -> String {
        if self.is_empty() {
            return String::new();
        }

        let start_offset = self.start.to_byte_offset(rope);
        let end_offset = self.end.to_byte_offset(rope);

        rope.slice(start_offset..end_offset).to_string()
    }

    /// Check if a position is within the selection
    pub fn contains(&self, position: Position) -> bool {
        if self.is_empty() {
            return false;
        }

        (position.line > self.start.line
            || (position.line == self.start.line && position.column >= self.start.column))
            && (position.line < self.end.line
                || (position.line == self.end.line && position.column <= self.end.column))
    }

    /// Extend the selection to include a new position
    pub fn extend_to(&mut self, position: Position) {
        if position.line < self.start.line
            || (position.line == self.start.line && position.column < self.start.column)
        {
            self.start = position;
        } else if position.line > self.end.line
            || (position.line == self.end.line && position.column > self.end.column)
        {
            self.end = position;
        }
    }

    /// Get byte offsets for the selection
    pub fn to_byte_range(&self, rope: &Rope) -> (usize, usize) {
        let start_offset = self.start.to_byte_offset(rope);
        let end_offset = self.end.to_byte_offset(rope);
        (start_offset, end_offset)
    }

    /// Create a selection for an entire line
    pub fn line(rope: &Rope, line_num: usize) -> Option<Self> {
        if line_num >= rope.len_lines() {
            return None;
        }

        let start = Position::new(line_num, 0);

        // Select the entire line including the newline character
        // For the last line, select to the end of the line content
        let end = if line_num + 1 < rope.len_lines() {
            // There's a next line, so select up to the start of the next line (including newline)
            Position::new(line_num + 1, 0)
        } else {
            // This is the last line, select to the end of the line content
            let line = rope.line(line_num);
            Position::new(line_num, line.len_chars())
        };

        Some(Self::new(start, end))
    }

    /// Create a selection for a word at the given position
    pub fn word_at(rope: &Rope, position: Position) -> Option<Self> {
        let offset = position.to_byte_offset(rope);
        if offset >= rope.len_bytes() {
            return None;
        }

        let text = rope.slice(..);
        let ch = text.char(offset);

        if ch.is_whitespace() || ch.is_ascii_punctuation() {
            return None;
        }

        // Find word start
        let mut start_offset = offset;
        while start_offset > 0 {
            let prev_ch = text.char(start_offset - 1);
            if prev_ch.is_whitespace() || prev_ch.is_ascii_punctuation() {
                break;
            }
            start_offset -= 1;
        }

        // Find word end
        let mut end_offset = offset;
        while end_offset < rope.len_bytes() {
            let ch = text.char(end_offset);
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                break;
            }
            end_offset += 1;
        }

        let start_pos = Position::from_byte_offset(rope, start_offset);
        let end_pos = Position::from_byte_offset(rope, end_offset);

        Some(Self::new(start_pos, end_pos))
    }

    /// Create a selection for the entire document
    pub fn all(rope: &Rope) -> Self {
        let start = Position::zero();
        let end = if rope.len_lines() > 0 {
            let last_line = rope.len_lines() - 1;
            let line = rope.line(last_line);
            Position::new(last_line, line.len_chars())
        } else {
            Position::zero()
        };

        Self::new(start, end)
    }
}
