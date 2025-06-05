// Re-export both core and ui crates for convenience
pub use icedit_core as core;
pub use icedit_ui as ui;

// Re-export commonly used types from both crates for backward compatibility
pub use icedit_core::{
    Buffer, Cursor, CursorMovement, Editor, EditorEvent, EditorMessage, EditorResponse, Position,
    Selection,
};

pub use icedit_ui::{KeyBinding, Shortcut, ShortcutManager, UIEditor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_integration() {
        // Test that we can use the core editor directly
        let mut core_editor = Editor::new();
        let response =
            core_editor.handle_message(EditorMessage::InsertText("Hello from core!".to_string()));
        assert!(matches!(response, EditorResponse::TextChanged));

        // Test that we can use the UI editor
        let mut ui_editor = UIEditor::new();
        let response =
            ui_editor.handle_message(EditorMessage::InsertText("Hello from UI!".to_string()));
        assert!(matches!(response, EditorResponse::TextChanged));

        // Test shortcut functionality
        let shortcuts = ui_editor.shortcut_manager();
        assert!(!shortcuts.get_bindings().is_empty());
    }
}
