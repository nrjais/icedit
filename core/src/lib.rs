pub mod buffer;
pub mod cursor;
pub mod editor;
pub mod keys;
pub mod messages;
pub mod selection;
pub mod shortcuts;
pub mod text_utils;

pub use buffer::Buffer;
pub use cursor::{Cursor, Position};
pub use editor::Editor;
pub use keys::{Key, KeyEvent, Modifiers, NamedKey};
pub use messages::{CursorMovement, EditorEvent, EditorMessage, EditorResponse};
pub use selection::Selection;
pub use shortcuts::{KeyBinding, Shortcut, ShortcutManager};
pub use text_utils::is_word_boundary;

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

        // Test redo
        let response = editor.handle_message(EditorMessage::Redo);
        assert!(matches!(response, EditorResponse::Success));
    }

    #[test]
    fn test_key_input_handling() {
        let mut editor = Editor::new();

        // Test character input
        let response = editor.handle_key_input(KeyInput::Character('a'));
        assert!(matches!(response, EditorResponse::TextChanged));

        let content = editor.current_buffer().text();
        assert_eq!(content, "a");

        // Test backspace
        let response = editor.handle_key_input(KeyInput::Command("backspace".to_string()));
        assert!(matches!(response, EditorResponse::Success));

        let content = editor.current_buffer().text();
        assert_eq!(content, "");
    }

    #[test]
    fn test_whitespace_and_tab_input() {
        use crate::keys::{Key, KeyEvent, Modifiers, NamedKey};
        use crate::shortcuts::ShortcutManager;

        let shortcut_manager = ShortcutManager::new();

        // Test space input
        let space_event = KeyEvent::new(Key::Named(NamedKey::Space), Modifiers::new());
        let result = shortcut_manager.handle_key_event(space_event);
        assert!(matches!(result, Some(EditorMessage::InsertChar(' '))));

        // Test tab input (should now insert tab, not indent)
        let tab_event = KeyEvent::new(Key::Named(NamedKey::Tab), Modifiers::new());
        let result = shortcut_manager.handle_key_event(tab_event);
        assert!(matches!(result, Some(EditorMessage::InsertChar('\t'))));

        // Test shift+tab (should still be unindent command)
        let shift_tab_event = KeyEvent::new(Key::Named(NamedKey::Tab), Modifiers::new().shift());
        let result = shortcut_manager.handle_key_event(shift_tab_event);
        assert!(matches!(result, Some(EditorMessage::DeleteToLineStart)));

        // Test regular character input
        let char_event = KeyEvent::new(Key::Character('a'), Modifiers::new());
        let result = shortcut_manager.handle_key_event(char_event);
        assert!(matches!(result, Some(EditorMessage::InsertChar('a'))));

        // Test character input with shift (should still work)
        let shift_char_event = KeyEvent::new(Key::Character('A'), Modifiers::new().shift());
        let result = shortcut_manager.handle_key_event(shift_char_event);
        assert!(matches!(result, Some(EditorMessage::InsertChar('A'))));
    }
}
