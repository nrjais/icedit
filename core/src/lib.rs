pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod messages;
pub mod selection;

pub use buffer::Buffer;
pub use cursor::{Cursor, Position};
pub use editor::Editor;
pub use messages::{CursorMovement, EditorEvent, EditorMessage, EditorResponse};
pub use selection::Selection;

/// Key event for widget integration
#[derive(Debug, Clone)]
pub enum KeyInput {
    /// Character to insert
    Character(char),
    /// Special key command (like arrow keys, ctrl+c, etc.)
    Command(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_editor_functionality() {
        let mut editor = Editor::new();

        // Test inserting text
        let response =
            editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));
        assert!(matches!(response, EditorResponse::TextChanged));

        // Test getting buffer content
        let content = editor.current_buffer().text();
        assert_eq!(content, "Hello, World!");

        // Test cursor movement
        let response = editor.handle_message(EditorMessage::MoveCursor(CursorMovement::Left));
        assert!(matches!(response, EditorResponse::CursorMoved(_)));

        // Test selection
        let response = editor.handle_message(EditorMessage::SelectAll);
        assert!(matches!(response, EditorResponse::SelectionChanged(_)));

        // Test copy
        let response = editor.handle_message(EditorMessage::Copy);
        assert!(matches!(response, EditorResponse::Success));

        // Verify clipboard has content (note: selection might not include the last character)
        assert!(editor.clipboard().starts_with("Hello, World"));

        // Test undo
        let response = editor.handle_message(EditorMessage::Undo);
        assert!(matches!(response, EditorResponse::Success));
    }

    #[test]
    fn test_key_input_handling() {
        let mut editor = Editor::new();

        // Test character input
        let response = editor.handle_key_input(KeyInput::Character('H'));
        assert!(matches!(response, EditorResponse::TextChanged));

        let response = editor.handle_key_input(KeyInput::Character('i'));
        assert!(matches!(response, EditorResponse::TextChanged));

        let content = editor.current_buffer().text();
        assert_eq!(content, "Hi");

        // Test command input
        let response = editor.handle_key_input(KeyInput::Command("left".to_string()));
        assert!(matches!(response, EditorResponse::CursorMoved(_)));

        // Insert another character to test cursor position
        let response = editor.handle_key_input(KeyInput::Character('!'));
        assert!(matches!(response, EditorResponse::TextChanged));

        let content = editor.current_buffer().text();
        assert_eq!(content, "H!i");
    }
}
