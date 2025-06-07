use crate::{Position, Selection};

/// All possible editor actions represented as messages
#[derive(Debug, Clone, PartialEq)]
pub enum EditorMessage {
    // Text manipulation
    InsertChar(char),
    InsertText(String),
    DeleteChar,
    DeleteCharBackward,
    DeleteLine,
    DeleteSelection,

    // Cursor movement
    MoveCursor(CursorMovement),
    MoveCursorTo(Position),

    // Cursor movement with selection handling
    MoveCursorWithSelection(CursorMovement),

    // Selection
    StartSelection,
    EndSelection,
    SelectAll,
    SelectLine,
    SelectWord,
    ClearSelection,

    // Editing operations
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,

    // Search and replace
    Find(String),
    FindNext,
    FindPrevious,
    Replace(String, String),
    ReplaceAll(String, String),

    // View operations
    Scroll(f32, f32),
    ScrollToLine(usize),
    UpdateViewport(f32, f32),

    // Custom commands
    Command(String, Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
    PageUp,
    PageDown,
}

/// Response from the editor after processing a message
#[derive(Debug, Clone, PartialEq)]
pub enum EditorResponse {
    Success,
    Error(String),
    TextChanged,
    CursorMoved(Position),
    SelectionChanged(Option<Selection>),
    SearchResult(Vec<Position>),
}

/// Event that can be sent to UI layers
#[derive(Debug, Clone, PartialEq)]
pub enum EditorEvent {
    TextChanged,
    CursorMoved(Position),
    SelectionChanged(Option<Selection>),
    StatusMessage(String),
    Error(String),
}
