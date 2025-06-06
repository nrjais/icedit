use crate::{
    messages::{CursorMovement, EditorEvent, EditorResponse},
    Buffer, Cursor, EditorMessage, Position, Selection, ShortcutEvent,
};

/// Information about a partially visible line for smooth scrolling
///
/// This struct contains information about lines that are partially visible
/// in the viewport, enabling smooth scrolling by rendering only the visible
/// portions of each line with proper clipping.
#[derive(Debug, Clone)]
pub struct PartialLineView {
    /// Line index in the buffer
    pub line_index: usize,
    /// Y position relative to viewport top (can be negative for partially visible lines)
    pub y_offset: f32,
    /// How much of the line is visible (0.0 to 1.0)
    pub visible_fraction: f32,
    /// The clipped area of the line that should be rendered
    pub clip_top: f32,
    pub clip_bottom: f32,
}

/// Viewport information for rendering optimization with partial line support
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Scroll offset (x, y) - supports fractional positions for smooth scrolling
    pub scroll_offset: (f32, f32),
    /// Viewport size (width, height)
    pub size: (f32, f32),
    /// Character dimensions for text layout
    pub char_width: f32,
    pub line_height: f32,
    /// Visible line range (start_line, end_line) - calculated based on scroll and viewport
    pub visible_lines: (usize, usize),
    /// Information about partially visible lines at top and bottom
    pub partial_lines: Vec<PartialLineView>,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            scroll_offset: (0.0, 0.0),
            size: (800.0, 600.0),
            char_width: 8.0,
            line_height: 18.0,
            visible_lines: (0, 0),
            partial_lines: Vec::new(),
        }
    }

    /// Update viewport size and recalculate visible lines
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size = (width, height);
        self.update_visible_lines();
    }

    /// Update scroll offset and recalculate visible lines
    pub fn set_scroll_offset(&mut self, x: f32, y: f32) {
        self.scroll_offset = (x, y);
        self.update_visible_lines();
    }

    /// Set character dimensions
    pub fn set_char_dimensions(&mut self, char_width: f32, line_height: f32) {
        self.char_width = char_width;
        self.line_height = line_height;
        self.update_visible_lines();
    }

    /// Calculate which lines are visible based on scroll and viewport, including partial lines
    fn update_visible_lines(&mut self) {
        let scroll_y = self.scroll_offset.1;
        let viewport_height = self.size.1;
        let line_height = self.line_height;

        // Calculate the range of lines that intersect with the viewport
        let first_line_top = (scroll_y / line_height).floor();
        let last_line_bottom = ((scroll_y + viewport_height) / line_height).ceil();

        let start_line = first_line_top as usize;
        let end_line = last_line_bottom as usize;

        self.visible_lines = (start_line, end_line);

        // Calculate partial line information
        self.partial_lines.clear();

        for line_idx in start_line..end_line {
            let line_y_top = line_idx as f32 * line_height;
            let line_y_bottom = line_y_top + line_height;
            let viewport_top = scroll_y;
            let viewport_bottom = scroll_y + viewport_height;

            // Check if line intersects with viewport
            if line_y_bottom > viewport_top && line_y_top < viewport_bottom {
                let y_offset = line_y_top - viewport_top;

                // Calculate clipping
                let clip_top = if line_y_top < viewport_top {
                    viewport_top - line_y_top
                } else {
                    0.0
                };

                let clip_bottom = if line_y_bottom > viewport_bottom {
                    line_y_bottom - viewport_bottom
                } else {
                    0.0
                };

                let visible_height = line_height - clip_top - clip_bottom;
                let visible_fraction = visible_height / line_height;

                // Only consider lines that have some visible portion
                if visible_fraction > 0.0 {
                    self.partial_lines.push(PartialLineView {
                        line_index: line_idx,
                        y_offset,
                        visible_fraction,
                        clip_top,
                        clip_bottom,
                    });
                }
            }
        }
    }

    /// Check if a line is currently visible
    pub fn is_line_visible(&self, line: usize) -> bool {
        line >= self.visible_lines.0 && line <= self.visible_lines.1
    }

    /// Get scroll bounds to prevent over-scrolling
    pub fn clamp_scroll_offset(&self, offset: (f32, f32), content_lines: usize) -> (f32, f32) {
        let (x, y) = offset;
        let content_height = content_lines as f32 * self.line_height;

        let clamped_x = x.max(0.0);
        let clamped_y = if content_height > self.size.1 {
            y.max(0.0).min(content_height - self.size.1)
        } else {
            0.0
        };

        (clamped_x, clamped_y)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

/// Main editor state and logic
pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    selection: Option<Selection>,
    clipboard: String,
    viewport: Viewport,
}

impl Editor {
    /// Create a new editor instance
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            selection: None,
            clipboard: String::new(),
            viewport: Viewport::new(),
        }
    }

    /// Create a new editor instance with text
    pub fn with_text(text: &str) -> Self {
        Self {
            buffer: Buffer::from_text(text),
            cursor: Cursor::new(),
            selection: None,
            clipboard: String::new(),
            viewport: Viewport::new(),
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

    /// Get the viewport
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Get mutable viewport
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Update viewport size
    pub fn set_viewport_size(&mut self, width: f32, height: f32) {
        self.viewport.set_size(width, height);
    }

    /// Update scroll offset with bounds checking
    pub fn set_scroll_offset(&mut self, x: f32, y: f32) {
        let line_count = self.buffer.line_count();
        let clamped_offset = self.viewport.clamp_scroll_offset((x, y), line_count);
        self.viewport
            .set_scroll_offset(clamped_offset.0, clamped_offset.1);
    }

    /// Update character dimensions
    pub fn set_char_dimensions(&mut self, char_width: f32, line_height: f32) {
        self.viewport.set_char_dimensions(char_width, line_height);
    }

    /// Get visible text lines for efficient rendering
    pub fn get_visible_lines(&self) -> Vec<String> {
        let rope = self.buffer.rope();
        let (start_line, end_line) = self.viewport.visible_lines;
        let total_lines = rope.len_lines();

        let mut lines = Vec::new();
        for line_idx in start_line..end_line.min(total_lines) {
            if let Some(line) = rope.get_line(line_idx) {
                lines.push(line.to_string());
            }
        }
        lines
    }

    /// Get visible lines with partial line information for smooth scrolling
    pub fn get_visible_lines_with_partial(&self) -> Vec<(String, &PartialLineView)> {
        let rope = self.buffer.rope();
        let total_lines = rope.len_lines();

        let mut lines_with_partial = Vec::new();
        for partial_line in &self.viewport.partial_lines {
            if partial_line.line_index < total_lines {
                if let Some(line) = rope.get_line(partial_line.line_index) {
                    lines_with_partial.push((line.to_string(), partial_line));
                }
            }
        }
        lines_with_partial
    }

    /// Check if the viewport has partial lines (indicating smooth scrolling is active)
    pub fn has_partial_lines(&self) -> bool {
        !self.viewport.partial_lines.is_empty()
    }

    /// Get the number of partial lines being rendered
    pub fn partial_line_count(&self) -> usize {
        self.viewport.partial_lines.len()
    }

    /// Check if cursor should be visible and auto-scroll if needed
    pub fn ensure_cursor_visible(&mut self) {
        let cursor_pos = self.cursor.position();
        let cursor_y = cursor_pos.line as f32 * self.viewport.line_height;
        let cursor_x = cursor_pos.column as f32 * self.viewport.char_width;

        let mut scroll_x = self.viewport.scroll_offset.0;
        let mut scroll_y = self.viewport.scroll_offset.1;
        let mut changed = false;

        // Check vertical scrolling
        if cursor_y < scroll_y {
            scroll_y = cursor_y;
            changed = true;
        } else if cursor_y + self.viewport.line_height > scroll_y + self.viewport.size.1 {
            scroll_y = cursor_y + self.viewport.line_height - self.viewport.size.1;
            changed = true;
        }

        // Check horizontal scrolling
        if cursor_x < scroll_x {
            scroll_x = cursor_x;
            changed = true;
        } else if cursor_x + self.viewport.char_width > scroll_x + self.viewport.size.0 {
            scroll_x = cursor_x + self.viewport.char_width - self.viewport.size.0;
            changed = true;
        }

        if changed {
            self.set_scroll_offset(scroll_x, scroll_y);
        }
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

            EditorMessage::MoveCursor(movement) => self.handle_cursor_movement(movement),
            EditorMessage::MoveCursorTo(position) => self.handle_move_cursor_to(position),
            EditorMessage::MoveCursorWithSelection(movement, extend_selection) => {
                self.handle_cursor_movement_with_selection(movement, extend_selection)
            }

            EditorMessage::StartSelection => self.handle_start_selection(),
            EditorMessage::EndSelection => self.handle_end_selection(),
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

            EditorMessage::ScrollUp(_) => EditorResponse::Success,
            EditorMessage::ScrollDown(_) => EditorResponse::Success,
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
            Ok(_) => {
                self.ensure_cursor_visible();
                EditorResponse::TextChanged
            }
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
            Ok(_) => {
                self.ensure_cursor_visible();
                EditorResponse::TextChanged
            }
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_char(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead of single character
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => {
                        self.ensure_cursor_visible();
                        return EditorResponse::Success;
                    }
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        let position = self.cursor.position();
        match self.buffer.delete_char(position, &mut self.cursor) {
            Ok(_) => {
                self.ensure_cursor_visible();
                EditorResponse::Success
            }
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_char_backward(&mut self) -> EditorResponse {
        // If there's a selection, delete it instead of single character
        if let Some(selection) = self.selection.take() {
            if !selection.is_empty() {
                match self.buffer.delete_selection(&selection, &mut self.cursor) {
                    Ok(_) => {
                        self.ensure_cursor_visible();
                        return EditorResponse::Success;
                    }
                    Err(e) => return EditorResponse::Error(e.to_string()),
                }
            }
        }

        let position = self.cursor.position();
        match self.buffer.delete_char_backward(position, &mut self.cursor) {
            Ok(_) => {
                self.ensure_cursor_visible();
                EditorResponse::Success
            }
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_line(&mut self) -> EditorResponse {
        let line = self.cursor.position().line;
        match self.buffer.delete_line(line, &mut self.cursor) {
            Ok(_) => {
                self.ensure_cursor_visible();
                EditorResponse::Success
            }
            Err(e) => EditorResponse::Error(e.to_string()),
        }
    }

    fn handle_delete_selection(&mut self) -> EditorResponse {
        if let Some(selection) = self.selection.take() {
            match self.buffer.delete_selection(&selection, &mut self.cursor) {
                Ok(_) => {
                    self.ensure_cursor_visible();
                    EditorResponse::Success
                }
                Err(e) => EditorResponse::Error(e.to_string()),
            }
        } else {
            EditorResponse::Success
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

            self.ensure_cursor_visible();
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
        self.ensure_cursor_visible();

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
        extend_selection: bool,
    ) -> EditorResponse {
        let rope = self.buffer.rope();
        let initial_position = self.cursor.position();

        // Track the anchor point for selection extension
        let selection_anchor = if extend_selection {
            if let Some(ref selection) = self.selection {
                // If cursor is at the start of selection, anchor is the end and vice versa
                if initial_position == selection.start {
                    selection.end
                } else {
                    selection.start
                }
            } else {
                // Starting a new selection, anchor is the current position
                initial_position
            }
        } else {
            initial_position
        };

        // If we're starting a selection and don't have one yet
        if extend_selection && self.selection.is_none() {
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
            self.ensure_cursor_visible();
            let new_position = self.cursor.position();

            if extend_selection {
                // Update selection to include new cursor position
                if let Some(ref mut selection) = self.selection {
                    // Use the proper anchor point for selection extension
                    *selection = Selection::from_positions(selection_anchor, new_position);
                }
                EditorResponse::SelectionChanged(self.selection.clone())
            } else {
                // Clear selection if not extending
                if self.selection.is_some() {
                    self.selection = None;
                }
                EditorResponse::CursorMoved(new_position)
            }
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

    /// Handle scroll operations with automatic viewport management
    pub fn handle_scroll(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> EditorResponse {
        // Update viewport size if provided
        self.set_viewport_size(viewport_width, viewport_height);

        // Handle scrolling using the core editor's viewport management
        let current_offset = self.viewport.scroll_offset;
        let new_offset = (current_offset.0 + delta_x, current_offset.1 + delta_y);

        // Clamp the scroll offset to valid bounds
        let line_count = self.buffer.line_count();
        let clamped_offset = self.viewport.clamp_scroll_offset(new_offset, line_count);
        self.set_scroll_offset(clamped_offset.0, clamped_offset.1);

        EditorResponse::Success
    }

    /// Handle mouse click at a specific position
    pub fn handle_mouse_click(&mut self, position: Position) -> EditorResponse {
        self.handle_message(EditorMessage::MoveCursorTo(position))
    }

    /// Handle shortcut events (keyboard shortcuts and character input)
    pub fn handle_shortcut_event(&mut self, shortcut_event: ShortcutEvent) -> EditorResponse {
        match shortcut_event {
            ShortcutEvent::EditorMessage(message) => self.handle_message(message),
            ShortcutEvent::CharacterInput(ch) => self.handle_message(EditorMessage::InsertChar(ch)),
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
