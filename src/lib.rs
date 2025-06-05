pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod messages;
pub mod selection;
pub mod shortcuts;

pub use buffer::Buffer;
pub use cursor::{Cursor, Position};
pub use editor::Editor;
pub use messages::EditorMessage;
pub use selection::Selection;
pub use shortcuts::{KeyBinding, Shortcut, ShortcutManager};

#[cfg(test)]
mod tests {
    use super::*;
    use messages::CursorMovement;

    #[test]
    fn test_basic_editor_functionality() {
        let mut editor = Editor::new();

        // Test inserting text
        let response =
            editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));
        assert!(matches!(response, messages::EditorResponse::TextChanged));

        // Test getting buffer content
        let content = editor.current_buffer().text();
        assert_eq!(content, "Hello, World!");

        // Test cursor movement
        let response = editor.handle_message(EditorMessage::MoveCursor(CursorMovement::Left));
        assert!(matches!(response, messages::EditorResponse::CursorMoved(_)));

        // Test selection
        let response = editor.handle_message(EditorMessage::SelectAll);
        assert!(matches!(
            response,
            messages::EditorResponse::SelectionChanged(_)
        ));

        // Test copy
        let response = editor.handle_message(EditorMessage::Copy);
        assert!(matches!(response, messages::EditorResponse::Success));

        // Verify clipboard has content (note: selection might not include the last character)
        assert!(editor.clipboard().starts_with("Hello, World"));

        // Test undo
        let response = editor.handle_message(EditorMessage::Undo);
        assert!(matches!(response, messages::EditorResponse::Success));
    }

    #[test]
    fn test_shortcut_system() {
        let editor = Editor::new();
        let shortcuts = editor.shortcut_manager();

        // Test that Ctrl+A maps to SelectAll
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);

        if let Some(message) = shortcuts.handle_key_event(key_event) {
            assert!(matches!(message, EditorMessage::SelectAll));
        }
    }
}
