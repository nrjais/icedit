use iced::{Element, Task};
use icedit_core::{Editor, EditorMessage, ShortcutEvent};
use icedit_ui::{styled_editor, WidgetMessage};

/// Test application to verify selection rendering sync
struct SelectionTestApp {
    editor: Editor,
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    Widget(WidgetMessage),
}

impl SelectionTestApp {
    fn new() -> Self {
        // Create a longer test text with many lines to test sync later in buffer
        let mut test_lines = Vec::new();
        for i in 0..200 {
            test_lines.push(format!("Line {} - This is a longer line with varied content to test selection rendering synchronization issues that might occur later in the buffer when scrolling down far enough", i + 1));
        }
        let test_text = test_lines.join("\n");

        let mut editor = Editor::with_text(&test_text);

        // Initialize viewport with reasonable defaults
        editor.set_viewport_size(800.0, 600.0);
        editor.set_char_dimensions(8.0, 18.0);

        // Start with a selection at the beginning
        let _ = editor.handle_message(EditorMessage::SelectAll);

        Self { editor }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Widget(widget_message) => {
                match widget_message {
                    WidgetMessage::ShortcutEvent(shortcut_event) => {
                        let _response = match shortcut_event {
                            ShortcutEvent::EditorMessage(message) => {
                                self.editor.handle_message(message)
                            }
                            ShortcutEvent::CharacterInput(ch) => {
                                self.editor.handle_message(EditorMessage::InsertChar(ch))
                            }
                        };
                    }
                    WidgetMessage::MousePressed(position) => {
                        let _response = self
                            .editor
                            .handle_message(EditorMessage::MoveCursorTo(position));
                    }
                    WidgetMessage::Scroll(delta, bounds) => {
                        let current_offset = self.editor.viewport().scroll_offset;
                        let new_offset = (current_offset.0 + delta.x, current_offset.1 + delta.y);
                        let clamped_offset = bounds
                            .clamp_scroll_offset(iced::Vector::new(new_offset.0, new_offset.1));
                        self.editor
                            .set_scroll_offset(clamped_offset.x, clamped_offset.y);
                    }
                    WidgetMessage::MouseReleased(_position) => {
                        // Handle mouse release events if needed
                    }
                    WidgetMessage::MouseMoved(_position) => {
                        // Handle mouse move events if needed
                    }
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        styled_editor(
            &self.editor,
            16.0,            // Font size
            true,            // Dark theme
            Message::Widget, // Message mapper
        )
    }
}

impl Default for SelectionTestApp {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> iced::Result {
    iced::run(
        "Selection Sync Test",
        SelectionTestApp::update,
        SelectionTestApp::view,
    )
}
