use iced::{Element, Task};
use icedit_core::{Editor, EditorMessage, ShortcutEvent};
use icedit_ui::{styled_editor, WidgetMessage};

/// Test application to verify line selection functionality
struct LineSelectionTestApp {
    editor: Editor,
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    Widget(WidgetMessage),
}

impl LineSelectionTestApp {
    fn new() -> Self {
        // Create test text with different line lengths to test selection
        let test_text = "Short line\nThis is a much longer line with more content to test horizontal selection rendering\nMedium length line here\nAnother line\n\nEmpty line above\nFinal line for testing";

        let mut editor = Editor::with_text(test_text);

        // Initialize viewport with reasonable defaults
        editor.set_viewport_size(800.0, 600.0);
        editor.set_char_dimensions(8.0, 18.0);

        // Start by selecting the second line to test line selection
        let _ = editor.handle_message(EditorMessage::MoveCursorTo(icedit_core::Position::new(
            1, 20,
        )));
        let _ = editor.handle_message(EditorMessage::SelectLine);

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

impl Default for LineSelectionTestApp {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> iced::Result {
    iced::run(
        "Line Selection Test",
        LineSelectionTestApp::update,
        LineSelectionTestApp::view,
    )
}
