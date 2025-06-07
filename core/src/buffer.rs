use crate::{Cursor, Position, Selection};
use ropey::Rope;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferError {
    #[error("Invalid position: line {line}, column {column}")]
    InvalidPosition { line: usize, column: usize },
}

/// Represents a text buffer with undo/redo capabilities
#[derive(Debug, Clone)]
pub struct Buffer {
    rope: Rope,
    is_modified: bool,
    undo_stack: Vec<BufferState>,
    redo_stack: Vec<BufferState>,
    max_undo_levels: usize,
}

#[derive(Debug, Clone)]
struct BufferState {
    rope: Rope,
    cursor_position: Position,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            is_modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: 100,
        }
    }

    /// Create a buffer from text content
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            is_modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_levels: 100,
        }
    }

    /// Get the rope reference
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Check if buffer is modified
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Get the entire text content
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get text for a specific line
    pub fn line_text(&self, line: usize) -> Option<String> {
        if line >= self.rope.len_lines() {
            return None;
        }
        Some(self.rope.line(line).to_string())
    }

    /// Get number of lines
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get number of characters
    pub fn char_count(&self) -> usize {
        self.rope.len_chars()
    }

    /// Save current state for undo
    fn save_state(&mut self, cursor_position: Position) {
        let state = BufferState {
            rope: self.rope.clone(),
            cursor_position,
        };

        self.undo_stack.push(state);
        if self.undo_stack.len() > self.max_undo_levels {
            self.undo_stack.remove(0);
        }

        // Clear redo stack when new action is performed
        self.redo_stack.clear();
    }

    /// Insert character at position
    pub fn insert_char(
        &mut self,
        position: Position,
        ch: char,
        cursor: &mut Cursor,
    ) -> Result<(), BufferError> {
        self.save_state(cursor.position());

        let offset = position.to_byte_offset(&self.rope);
        self.rope.insert_char(offset, ch);
        self.is_modified = true;

        // Move cursor after inserted character
        cursor.set_position(Position::from_byte_offset(
            &self.rope,
            offset + ch.len_utf8(),
        ));

        Ok(())
    }

    /// Insert text at position
    pub fn insert_text(
        &mut self,
        position: Position,
        text: &str,
        cursor: &mut Cursor,
    ) -> Result<(), BufferError> {
        if text.is_empty() {
            return Ok(());
        }

        self.save_state(cursor.position());

        let offset = position.to_byte_offset(&self.rope);
        self.rope.insert(offset, text);
        self.is_modified = true;

        // Move cursor after inserted text
        cursor.set_position(Position::from_byte_offset(&self.rope, offset + text.len()));

        Ok(())
    }

    /// Delete character at position
    pub fn delete_char(
        &mut self,
        position: Position,
        cursor: &mut Cursor,
    ) -> Result<bool, BufferError> {
        let offset = position.to_byte_offset(&self.rope);
        if offset >= self.rope.len_bytes() {
            return Ok(false);
        }

        self.save_state(cursor.position());

        let ch = self.rope.char(offset);
        self.rope.remove(offset..offset + ch.len_utf8());
        self.is_modified = true;

        Ok(true)
    }

    /// Delete character before position (backspace)
    pub fn delete_char_backward(
        &mut self,
        position: Position,
        cursor: &mut Cursor,
    ) -> Result<bool, BufferError> {
        let offset = position.to_byte_offset(&self.rope);
        if offset == 0 {
            return Ok(false);
        }

        self.save_state(cursor.position());

        // Find the previous character boundary
        let text = self.rope.slice(..);
        let mut char_idx = text.byte_to_char(offset);
        if char_idx > 0 {
            char_idx -= 1;
        }
        let prev_offset = text.char_to_byte(char_idx);

        self.rope.remove(prev_offset..offset);
        self.is_modified = true;

        // Move cursor to deletion point
        cursor.set_position(Position::from_byte_offset(&self.rope, prev_offset));

        Ok(true)
    }

    /// Delete entire line
    pub fn delete_line(&mut self, line: usize, cursor: &mut Cursor) -> Result<bool, BufferError> {
        if line >= self.rope.len_lines() {
            return Ok(false);
        }

        self.save_state(cursor.position());

        let line_start = self.rope.line_to_byte(line);
        let line_end = if line + 1 < self.rope.len_lines() {
            self.rope.line_to_byte(line + 1)
        } else {
            self.rope.len_bytes()
        };

        self.rope.remove(line_start..line_end);
        self.is_modified = true;

        // Move cursor to start of line (or previous line if deleted last line)
        let new_line = if line < self.rope.len_lines() {
            line
        } else {
            self.rope.len_lines().saturating_sub(1)
        };
        cursor.set_position(Position::new(new_line, 0));

        Ok(true)
    }

    /// Delete selected text
    pub fn delete_selection(
        &mut self,
        selection: &Selection,
        cursor: &mut Cursor,
    ) -> Result<String, BufferError> {
        if selection.is_empty() {
            return Ok(String::new());
        }

        self.save_state(cursor.position());

        let (start_offset, end_offset) = selection.to_byte_range(&self.rope);
        let deleted_text = self.rope.slice(start_offset..end_offset).to_string();

        self.rope.remove(start_offset..end_offset);
        self.is_modified = true;

        // Move cursor to start of deleted selection
        cursor.set_position(selection.start);

        Ok(deleted_text)
    }

    /// Delete word forward (from cursor position to end of current word)
    pub fn delete_word_forward(&mut self, cursor: &mut Cursor) -> Result<bool, BufferError> {
        let current_pos = cursor.position();
        let current_offset = current_pos.to_byte_offset(&self.rope);

        if current_offset >= self.rope.len_bytes() {
            return Ok(false);
        }

        self.save_state(current_pos);

        let text = self.rope.slice(..);
        let mut end_offset = current_offset;

        // Skip current word (non-boundary characters)
        while end_offset < self.rope.len_bytes() {
            let ch = text.char(end_offset);
            if crate::text_utils::is_word_boundary(ch) {
                break;
            }
            end_offset += 1;
        }

        // Skip boundaries (whitespace and punctuation) until we find a word character or end
        while end_offset < self.rope.len_bytes() {
            let ch = text.char(end_offset);
            if !crate::text_utils::is_word_boundary(ch) {
                break;
            }
            end_offset += 1;
        }

        if end_offset > current_offset {
            self.rope.remove(current_offset..end_offset);
            self.is_modified = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete word backward (from cursor position to beginning of current word)
    pub fn delete_word_backward(&mut self, cursor: &mut Cursor) -> Result<bool, BufferError> {
        let current_pos = cursor.position();
        let current_offset = current_pos.to_byte_offset(&self.rope);

        if current_offset == 0 {
            return Ok(false);
        }

        self.save_state(current_pos);

        let text = self.rope.slice(..);
        let mut start_offset = current_offset;

        // Skip boundaries (whitespace and punctuation) backwards until we find a word character
        while start_offset > 0 {
            let ch = text.char(start_offset - 1);
            if !crate::text_utils::is_word_boundary(ch) {
                break;
            }
            start_offset -= 1;
        }

        // Skip word characters backwards to find the beginning of the word
        while start_offset > 0 {
            let ch = text.char(start_offset - 1);
            if crate::text_utils::is_word_boundary(ch) {
                break;
            }
            start_offset -= 1;
        }

        if start_offset < current_offset {
            self.rope.remove(start_offset..current_offset);
            self.is_modified = true;
            // Move cursor to deletion point
            cursor.set_position(Position::from_byte_offset(&self.rope, start_offset));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete from cursor to end of line
    pub fn delete_to_line_end(&mut self, cursor: &mut Cursor) -> Result<bool, BufferError> {
        let current_pos = cursor.position();
        let line = current_pos.line;

        if line >= self.rope.len_lines() {
            return Ok(false);
        }

        self.save_state(current_pos);

        let _line_start = self.rope.line_to_byte(line);
        let line_end = if line + 1 < self.rope.len_lines() {
            self.rope.line_to_byte(line + 1) - 1 // Don't include the newline
        } else {
            self.rope.len_bytes()
        };

        let current_offset = current_pos.to_byte_offset(&self.rope);

        if current_offset < line_end {
            self.rope.remove(current_offset..line_end);
            self.is_modified = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete from cursor to beginning of line
    pub fn delete_to_line_start(&mut self, cursor: &mut Cursor) -> Result<bool, BufferError> {
        let current_pos = cursor.position();
        let line = current_pos.line;

        if line >= self.rope.len_lines() {
            return Ok(false);
        }

        self.save_state(current_pos);

        let line_start = self.rope.line_to_byte(line);
        let current_offset = current_pos.to_byte_offset(&self.rope);

        if current_offset > line_start {
            self.rope.remove(line_start..current_offset);
            self.is_modified = true;
            // Move cursor to beginning of line
            cursor.set_position(Position::new(line, 0));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Undo last operation
    pub fn undo(&mut self, cursor: &mut Cursor) -> Result<bool, BufferError> {
        if let Some(state) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current_state = BufferState {
                rope: self.rope.clone(),
                cursor_position: cursor.position(),
            };
            self.redo_stack.push(current_state);

            // Restore previous state
            self.rope = state.rope;
            cursor.set_position(state.cursor_position);
            self.is_modified = true;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Redo last undone operation
    pub fn redo(&mut self, cursor: &mut Cursor) -> Result<bool, BufferError> {
        if let Some(state) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current_state = BufferState {
                rope: self.rope.clone(),
                cursor_position: cursor.position(),
            };
            self.undo_stack.push(current_state);

            // Restore redo state
            self.rope = state.rope;
            cursor.set_position(state.cursor_position);
            self.is_modified = true;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Find all occurrences of a pattern
    pub fn find(&self, pattern: &str) -> Vec<Position> {
        let mut positions = Vec::new();
        let text = self.rope.to_string();

        for (line_idx, line) in text.lines().enumerate() {
            let mut start = 0;
            while let Some(pos) = line[start..].find(pattern) {
                positions.push(Position::new(line_idx, start + pos));
                start += pos + 1;
            }
        }

        positions
    }

    /// Replace all occurrences of a pattern
    pub fn replace_all(
        &mut self,
        pattern: &str,
        replacement: &str,
        cursor: &mut Cursor,
    ) -> Result<usize, BufferError> {
        let text = self.rope.to_string();
        let new_text = text.replace(pattern, replacement);

        if text != new_text {
            self.save_state(cursor.position());
            self.rope = Rope::from_str(&new_text);
            self.is_modified = true;

            // Count replacements
            let count = text.matches(pattern).count();
            Ok(count)
        } else {
            Ok(0)
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
