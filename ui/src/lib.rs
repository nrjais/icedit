pub mod shortcuts;
pub mod widget;

// Re-export core types for convenience
pub use icedit_core::*;

// Export UI-specific types
pub use shortcuts::{KeyBinding, Shortcut, ShortcutManager};
pub use widget::*;

/// UI Editor that combines core editor with shortcut management
pub struct UIEditor {
    core_editor: Editor,
    shortcut_manager: ShortcutManager,
}

impl UIEditor {
    /// Create a new UI editor instance
    pub fn new() -> Self {
        Self {
            core_editor: Editor::new(),
            shortcut_manager: ShortcutManager::new(),
        }
    }

    /// Create a new UI editor instance with text
    pub fn with_text(text: &str) -> Self {
        Self {
            core_editor: Editor::with_text(text),
            shortcut_manager: ShortcutManager::new(),
        }
    }

    /// Get the core editor
    pub fn core_editor(&self) -> &Editor {
        &self.core_editor
    }

    /// Get the core editor mutably
    pub fn core_editor_mut(&mut self) -> &mut Editor {
        &mut self.core_editor
    }

    /// Get the shortcut manager
    pub fn shortcut_manager(&self) -> &ShortcutManager {
        &self.shortcut_manager
    }

    /// Get the shortcut manager mutably
    pub fn shortcut_manager_mut(&mut self) -> &mut ShortcutManager {
        &mut self.shortcut_manager
    }

    /// Handle a key input using shortcuts, returning the editor response if a shortcut was triggered
    pub fn handle_key_input(&mut self, input: KeyInput) -> Option<EditorResponse> {
        if let Some(message) = self.shortcut_manager.handle_key_input(&input) {
            Some(self.core_editor.handle_message(message))
        } else {
            // If no shortcut matched, handle the input directly
            Some(self.core_editor.handle_key_input(input))
        }
    }

    /// Process an editor message directly
    pub fn handle_message(&mut self, message: EditorMessage) -> EditorResponse {
        self.core_editor.handle_message(message)
    }

    /// Add an event handler
    pub fn add_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(&EditorEvent) + Send + 'static,
    {
        self.core_editor.add_event_handler(handler);
    }

    /// Get the current buffer
    pub fn current_buffer(&self) -> &Buffer {
        self.core_editor.current_buffer()
    }

    /// Get the current cursor
    pub fn current_cursor(&self) -> &Cursor {
        self.core_editor.current_cursor()
    }

    /// Get the current selection
    pub fn current_selection(&self) -> Option<&Selection> {
        self.core_editor.current_selection()
    }

    /// Get clipboard content
    pub fn clipboard(&self) -> &str {
        self.core_editor.clipboard()
    }

    /// Clear the editor content
    pub fn clear(&mut self) {
        self.core_editor.clear();
    }

    /// Set the editor content
    pub fn set_text(&mut self, text: &str) {
        self.core_editor.set_text(text);
    }
}

impl Default for UIEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_editor_functionality() {
        let mut editor = UIEditor::new();

        // Test core functionality still works
        let response =
            editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));
        assert!(matches!(response, EditorResponse::TextChanged));

        let content = editor.current_buffer().text();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_shortcut_integration() {
        let mut editor = UIEditor::new();

        // Insert some text first
        editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));

        // Test Ctrl+A shortcut for SelectAll
        let key_input = KeyInput::Command("ctrl+a".to_string());

        if let Some(response) = editor.handle_key_input(key_input) {
            assert!(matches!(response, EditorResponse::SelectionChanged(_)));
        }

        // Test copy shortcut
        let copy_input = KeyInput::Command("ctrl+c".to_string());
        if let Some(response) = editor.handle_key_input(copy_input) {
            assert!(matches!(response, EditorResponse::Success));
        }

        // Check clipboard has content
        assert!(editor.clipboard().starts_with("Hello, World"));
    }
}
