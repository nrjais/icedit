use iced::{Element, Task, Theme};
use icedit_core::Editor;
use icedit_ui::{styled_editor, EditorState, WidgetMessage};

/// Main application state
struct EditorApp {
    editor: Editor,
    editor_state: EditorState,
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    /// Widget messages from the editor
    Widget(WidgetMessage),
}

impl EditorApp {
    fn new() -> Self {
        let editor = Editor::with_text("Welcome to IcEdit!\n\nThis is a text editor built with Iced.\nYou can:\n- Type to insert text\n- Use arrow keys to move the cursor\n- Use Ctrl+A to select all\n- Use Ctrl+C/V for copy/paste\n- Use Ctrl+Z/Y for undo/redo\n- Scroll with mouse wheel");
        let editor_state = EditorState::from_editor(&editor);

        Self {
            editor,
            editor_state,
        }
    }

    fn title(&self) -> String {
        "IcEdit - Iced Text Editor".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Widget(widget_message) => {
                match widget_message {
                    WidgetMessage::Editor(editor_message) => {
                        // Handle editor messages by updating the core editor
                        let _response = self.editor.handle_message(editor_message);

                        // Update the widget state based on the editor's current state
                        self.update_editor_state();
                    }
                    WidgetMessage::Scroll(delta) => {
                        // Handle scrolling by updating the scroll offset
                        let mut new_offset = self.editor_state.scroll_offset + delta;

                        // Clamp scroll offset to reasonable bounds
                        new_offset.y = new_offset.y.max(0.0);
                        new_offset.x = new_offset.x.max(0.0);

                        self.editor_state.scroll_offset = new_offset;
                    }
                    WidgetMessage::MousePressed(_) => {
                        // Handle mouse press events if needed
                    }
                    WidgetMessage::MouseReleased(_) => {
                        // Handle mouse release events if needed
                    }
                    WidgetMessage::MouseMoved(_) => {
                        // Handle mouse move events if needed
                    }
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // Create the editor widget with current state
        styled_editor(
            self.editor_state.clone(),
            16.0,            // Font size
            true,            // Dark theme
            Message::Widget, // Message mapper
        )
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Update the editor state from the core editor
    fn update_editor_state(&mut self) {
        let old_scroll = self.editor_state.scroll_offset;
        self.editor_state = EditorState::from_editor(&self.editor);
        // Preserve scroll offset
        self.editor_state.scroll_offset = old_scroll;
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> iced::Result {
    iced::run("IcEdit", EditorApp::update, EditorApp::view)
}
