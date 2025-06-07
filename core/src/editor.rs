use crate::{
    messages::{CursorMovement, EditorEvent, EditorResponse},
    Buffer, Cursor, EditorMessage, Position, Selection,
};

/// Main editor state and logic
pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    selection: Option<Selection>,
    clipboard: String,
}

impl Editor {
    /// Create a new editor instance
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            selection: None,
            clipboard: String::new(),
        }
    }

    /// Create a new editor instance with text
    pub fn with_text(text: &str) -> Self {
        Self {
            buffer: Buffer::from_text(text),
            cursor: Cursor::new(),
            selection: None,
            clipboard: String::new(),
        }
    }

    /// Get the current buffer
    pub fn current_buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get the current cursor
    pub fn current_cursor(&self) -> &Cursor {
        &self.cursor
    }

    /// Get the current selection
    pub fn current_selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// Add an event handler (simplified version)
    pub fn add_event_handler<F>(&mut self, _handler: F)
    where
        F: Fn(&EditorEvent) + Send + 'static,
    {
        // In a real implementation, this would store the handler
        // For now, we'll just accept it to maintain the API
    }

    /// Process an editor message and return the response
    pub fn handle_message(&mut self, message: EditorMessage) -> EditorResponse {
        match message {
            EditorMessage::InsertChar(ch) => self.handle_insert_char(ch),
            EditorMessage::InsertText(text) => self.handle_insert_text(text),
            EditorMessage::DeleteChar => self.handle_delete_char(),
            EditorMessage::DeleteCharBackward => self.handle_delete_char_backward(),
            EditorMessage::DeleteLine => self.handle_delete_line(),
            EditorMessage::DeleteSelection => self.handle_delete_selection(),
            EditorMessage::DeleteWordForward => self.handle_delete_word_forward(),
            EditorMessage::DeleteWordBackward => self.handle_delete_word_backward(),
            EditorMessage::DeleteToLineEnd => self.handle_delete_to_line_end(),
            EditorMessage::DeleteToLineStart => self.handle_delete_to_line_start(),

            EditorMessage::MoveCursor(movement) => self.handle_cursor_movement(movement),
            EditorMessage::MoveCursorTo(position) => self.handle_move_cursor_to(position),
            EditorMessage::MoveCursorWithSelection(movement) => {
                self.handle_cursor_movement_with_selection(movement)
            }

            EditorMessage::StartSelection => self.handle_start_selection(),
            EditorMessage::EndSelection => self.handle_end_selection(),
            EditorMessage::SetSelection(start, end) => self.handle_set_selection(start, end),
            EditorMessage::SelectAll => self.handle_select_all(),
            EditorMessage::SelectLine => self.handle_select_line(),
            EditorMessage::SelectWord => self.handle_select_word(),
            EditorMessage::ClearSelection => self.handle_clear_selection(),

            EditorMessage::Undo => self.handle_undo(),
            EditorMessage::Redo => self.handle_redo(),
            EditorMessage::Cut => self.handle_cut(),
            EditorMessage::Copy => self.handle_copy(),
            EditorMessage::Paste => self.handle_paste(),

            EditorMessage::Find(pattern) => self.handle_find(pattern),
            EditorMessage::FindNext => EditorResponse::Success,
            EditorMessage::FindPrevious => EditorResponse::Success,
            EditorMessage::Replace(_, _) => EditorResponse::Success,
            EditorMessage::ReplaceAll(pattern, replacement) => {
                self.handle_replace_all(pattern, replacement)
            }

            EditorMessage::ScrollToLine(line) => self.handle_scroll_to_line(line),

            EditorMessage::Command(_, _) => EditorResponse::Success,
        }
    }

    // Text manipulation handlers
    fn handle_insert_char(&mut self, ch: char) -> EditorResponse {
        let position = self.cursor.position();

        // Delete selection if exists
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                let _ = self.buffer.delete_selection(&selection, &mut self.cursor);
            }
        }

        match self.buffer.insert_char(position, ch, &mut self.cursor) {
            Ok(_) => EditorResponse::TextChanged,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_insert_text(&mut self, text: String) -> EditorResponse {
        let position = self.cursor.position();

        // Delete selection if exists
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                let _ = self.buffer.delete_selection(&selection, &mut self.cursor);
            }
        }

        match self.buffer.insert_text(position, &text, &mut self.cursor) {
            Ok(_) => EditorResponse::TextChanged,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_char(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead of single character
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => {
                        return EditorResponse::Success;
                    }
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        let position = self.cursor.position();
        match self.buffer.delete_char(position, &mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_char_backward(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead of single character
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => {
                        return EditorResponse::Success;
                    }
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        let position = self.cursor.position();
        match self.buffer.delete_char_backward(position, &mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_line(&mut self) -> EditorResponse {
        let line = self.cursor.position().line;
        match self.buffer.delete_line(line, &mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_selection(&mut self) -> EditorResponse {
        if let Some(selection) = self.selection.take() {
            match self.buffer.delete_selection(&selection, &mut self.cursor) {
                Ok(_) => EditorResponse::Success,
                Err(e) => EditorResponse::Error(e.to_string()),
            }
        } else {
            EditorResponse::Success
        }
    }

    fn handle_delete_word_forward(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead of word
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => return EditorResponse::Success,
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        match self.buffer.delete_word_forward(&mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_word_backward(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead of word
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => return EditorResponse::Success,
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        match self.buffer.delete_word_backward(&mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_to_line_end(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => return EditorResponse::Success,
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        match self.buffer.delete_to_line_end(&mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_to_line_start(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => return EditorResponse::Success,
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        match self.buffer.delete_to_line_start(&mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    // Cursor movement handlers
    fn handle_cursor_movement(&mut self, movement: CursorMovement) -> EditorResponse {
        let rope = self.buffer.rope();
        let moved = match movement {
            CursorMovement::Up => self.cursor.move_up(rope),
            CursorMovement::Down => self.cursor.move_down(rope),
            CursorMovement::Left => self.cursor.move_left(rope),
            CursorMovement::Right => self.cursor.move_right(rope),
            CursorMovement::WordLeft => self.cursor.move_word_left(rope),
            CursorMovement::WordRight => self.cursor.move_word_right(rope),
            CursorMovement::LineStart => {
                self.cursor.move_to_line_start();
                true
            }
            CursorMovement::LineEnd => {
                self.cursor.move_to_line_end(rope);
                true
            }
            CursorMovement::DocumentStart => {
                self.cursor.move_to_document_start();
                true
            }
            CursorMovement::DocumentEnd => {
                self.cursor.move_to_document_end(rope);
                true
            }
            CursorMovement::PageUp => {
                for _ in 0..20 {
                    if !self.cursor.move_up(rope) {
                        break;
                    }
                }
                true
            }
            CursorMovement::PageDown => {
                for _ in 0..20 {
                    if !self.cursor.move_down(rope) {
                        break;
                    }
                }
                true
            }
        };

        if moved {
            // Clear any existing selection when moving cursor without extending selection
            let selection_cleared = self.selection.is_some();
            if selection_cleared {
                self.selection = None;
            }

            let position = self.cursor.position();

            // Return appropriate response based on whether selection was cleared
            if selection_cleared {
                EditorResponse::SelectionChanged(None)
            } else {
                EditorResponse::CursorMoved(position)
            }
        } else {
            EditorResponse::Success
        }
    }

    fn handle_move_cursor_to(&mut self, position: Position) -> EditorResponse {
        // Clear any existing selection when moving cursor to a specific position
        let selection_cleared = self.selection.is_some();
        if selection_cleared {
            self.selection = None;
        }

        self.cursor.set_position(position);

        // Return appropriate response based on whether selection was cleared
        if selection_cleared {
            EditorResponse::SelectionChanged(None)
        } else {
            EditorResponse::CursorMoved(position)
        }
    }

    fn handle_cursor_movement_with_selection(
        &mut self,
        movement: CursorMovement,
    ) -> EditorResponse {
        let rope = self.buffer.rope();
        let initial_position = self.cursor.position();

        // Track the anchor point for selection extension
        let selection_anchor = if let Some(ref selection) = self.selection {
            // If cursor is at the start of selection, anchor is the end and vice versa
            if initial_position == selection.start {
                selection.end
            } else {
                selection.start
            }
        } else {
            // Starting a new selection, anchor is the current position
            initial_position
        };

        // If we're starting a selection and don't have one yet
        if self.selection.is_none() {
            self.selection = Some(Selection::new(initial_position, initial_position));
        }

        let moved = match movement {
            CursorMovement::Up => self.cursor.move_up(rope),
            CursorMovement::Down => self.cursor.move_down(rope),
            CursorMovement::Left => self.cursor.move_left(rope),
            CursorMovement::Right => self.cursor.move_right(rope),
            CursorMovement::WordLeft => self.cursor.move_word_left(rope),
            CursorMovement::WordRight => self.cursor.move_word_right(rope),
            CursorMovement::LineStart => {
                self.cursor.move_to_line_start();
                true
            }
            CursorMovement::LineEnd => {
                self.cursor.move_to_line_end(rope);
                true
            }
            CursorMovement::DocumentStart => {
                self.cursor.move_to_document_start();
                true
            }
            CursorMovement::DocumentEnd => {
                self.cursor.move_to_document_end(rope);
                true
            }
            CursorMovement::PageUp => {
                for _ in 0..20 {
                    if !self.cursor.move_up(rope) {
                        break;
                    }
                }
                true
            }
            CursorMovement::PageDown => {
                for _ in 0..20 {
                    if !self.cursor.move_down(rope) {
                        break;
                    }
                }
                true
            }
        };

        if moved {
            let new_position = self.cursor.position();

            // Update selection to include new cursor position
            if let Some(ref mut selection) = self.selection {
                // Use the proper anchor point for selection extension
                *selection = Selection::from_positions(selection_anchor, new_position);
            }
            EditorResponse::SelectionChanged(self.selection.clone())
        } else {
            EditorResponse::Success
        }
    }

    // Selection handlers
    fn handle_start_selection(&mut self) -> EditorResponse {
        let position = self.cursor.position();
        self.selection = Some(Selection::new(position, position));
        EditorResponse::Success
    }

    fn handle_end_selection(&mut self) -> EditorResponse {
        if let Some(ref mut selection) = self.selection {
            selection.end = self.cursor.position();
            EditorResponse::SelectionChanged(Some(selection.clone()))
        } else {
            EditorResponse::Success
        }
    }

    fn handle_set_selection(&mut self, start: Position, end: Position) -> EditorResponse {
        let selection = Selection::from_positions(start, end);
        self.selection = Some(selection.clone());
        // Set cursor to the end position of the selection
        self.cursor.set_position(end);
        EditorResponse::SelectionChanged(Some(selection))
    }

    fn handle_select_all(&mut self) -> EditorResponse {
        let rope = self.buffer.rope();
        let selection = Selection::all(rope);
        self.selection = Some(selection.clone());
        EditorResponse::SelectionChanged(Some(selection))
    }

    fn handle_select_line(&mut self) -> EditorResponse {
        let rope = self.buffer.rope();
        let line = self.cursor.position().line;
        if let Some(selection) = Selection::line(rope, line) {
            self.selection = Some(selection.clone());
            EditorResponse::SelectionChanged(Some(selection))
        } else {
            EditorResponse::Error("Invalid line".to_string())
        }
    }

    fn handle_select_word(&mut self) -> EditorResponse {
        let rope = self.buffer.rope();
        let position = self.cursor.position();
        if let Some(selection) = Selection::word_at(rope, position) {
            self.selection = Some(selection.clone());
            EditorResponse::SelectionChanged(Some(selection))
        } else {
            EditorResponse::Success
        }
    }

    fn handle_clear_selection(&mut self) -> EditorResponse {
        self.selection = None;
        EditorResponse::SelectionChanged(None)
    }

    // Edit operation handlers
    fn handle_undo(&mut self) -> EditorResponse {
        match self.buffer.undo(&mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_redo(&mut self) -> EditorResponse {
        match self.buffer.redo(&mut self.cursor) {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_cut(&mut self) -> EditorResponse {
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                let text = selection.get_text(self.buffer.rope());
                self.clipboard = text;

                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => EditorResponse::Success,
                    Err(e) => EditorResponse::Error(e.to_string()),
                }
            } else {
                EditorResponse::Success
            }
        } else {
            EditorResponse::Success
        }
    }

    fn handle_copy(&mut self) -> EditorResponse {
        if let Some(selection) = &self.selection {
            if !selection.is_empty() {
                let text = selection.get_text(self.buffer.rope());
                self.clipboard = text;
            }
        }
        EditorResponse::Success
    }

    fn handle_paste(&mut self) -> EditorResponse {
        if !self.clipboard.is_empty() {
            self.handle_insert_text(self.clipboard.clone())
        } else {
            EditorResponse::Success
        }
    }

    // Search handlers
    fn handle_find(&mut self, pattern: String) -> EditorResponse {
        let results = self.buffer.find(&pattern);
        EditorResponse::SearchResult(results)
    }

    fn handle_replace_all(&mut self, pattern: String, replacement: String) -> EditorResponse {
        match self
            .buffer
            .replace_all(&pattern, &replacement, &mut self.cursor)
        {
            Ok(_) => EditorResponse::Success,
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_scroll_to_line(&mut self, line: usize) -> EditorResponse {
        let position = Position::new(line, 0);
        self.cursor.set_position(position);
        EditorResponse::CursorMoved(position)
    }

    /// Get clipboard content
    pub fn clipboard(&self) -> &str {
        &self.clipboard
    }

    /// Clear the editor content
    pub fn clear(&mut self) {
        self.buffer = Buffer::new();
        self.cursor = Cursor::new();
        self.selection = None;
    }

    /// Set the editor content
    pub fn set_text(&mut self, text: &str) {
        self.buffer = Buffer::from_text(text);
        self.cursor = Cursor::new();
        self.selection = None;
    }

    /// Handle key input from widgets - simplified interface
    pub fn handle_key_input(&mut self, input: crate::KeyInput) -> EditorResponse {
        match input {
            crate::KeyInput::Character(ch) => self.handle_message(EditorMessage::InsertChar(ch)),
            crate::KeyInput::Command(cmd) => {
                // Handle common commands
                match cmd.as_str() {
                    "backspace" => self.handle_message(EditorMessage::DeleteCharBackward),
                    "delete" => self.handle_message(EditorMessage::DeleteChar),
                    "left" => self.handle_message(EditorMessage::MoveCursor(CursorMovement::Left)),
                    "right" => {
                        self.handle_message(EditorMessage::MoveCursor(CursorMovement::Right))
                    }
                    "up" => self.handle_message(EditorMessage::MoveCursor(CursorMovement::Up)),
                    "down" => self.handle_message(EditorMessage::MoveCursor(CursorMovement::Down)),
                    "home" => {
                        self.handle_message(EditorMessage::MoveCursor(CursorMovement::LineStart))
                    }
                    "end" => {
                        self.handle_message(EditorMessage::MoveCursor(CursorMovement::LineEnd))
                    }
                    "ctrl+a" => self.handle_message(EditorMessage::SelectAll),
                    "ctrl+c" => self.handle_message(EditorMessage::Copy),
                    "ctrl+v" => self.handle_message(EditorMessage::Paste),
                    "ctrl+x" => self.handle_message(EditorMessage::Cut),
                    "ctrl+z" => self.handle_message(EditorMessage::Undo),
                    "ctrl+y" => self.handle_message(EditorMessage::Redo),
                    _ => EditorResponse::Success, // Unknown command
                }
            }
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
