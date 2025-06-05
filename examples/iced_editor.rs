use std::fs;

use iced::{Element, Task, Theme};
use icedit_core::{Editor, EditorMessage, ShortcutEvent};
use icedit_ui::{styled_editor, WidgetMessage};

/// Main application state
struct EditorApp {
    editor: Editor,
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    /// Widget messages from the editor
    Widget(WidgetMessage),
}

impl EditorApp {
    fn new() -> Self {
        let text = fs::read_to_string("README.md").unwrap();
        let mut editor = Editor::with_text(&text);

        // Initialize viewport with reasonable defaults
        editor.set_viewport_size(800.0, 600.0);
        editor.set_char_dimensions(8.0, 18.0);

        Self { editor }
    }

    fn title(&self) -> String {
        "IcEdit - Iced Text Editor".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Widget(widget_message) => {
                match widget_message {
                    WidgetMessage::ShortcutEvent(shortcut_event) => {
                        // Handle shortcut event using the core editor
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
                        // Position is already converted by the widget
                        let _response = self
                            .editor
                            .handle_message(EditorMessage::MoveCursorTo(position));
                    }
                    WidgetMessage::Scroll(delta, bounds) => {
                        // Handle scrolling using the core editor's viewport management
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
        // Create the editor widget with core editor reference
        styled_editor(
            &self.editor,
            16.0,            // Font size
            true,            // Dark theme
            Message::Widget, // Message mapper
        )
    }

    fn theme(&self) -> Theme {
        Theme::Dark
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
