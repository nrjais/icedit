use std::fs;

use iced::{Element, Task, Theme};
use icedit_core::Editor;
use icedit_ui::{get_char_dimensions, styled_editor, WidgetMessage};

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

        // Get measured character dimensions for the font size we'll use
        let font_size = 16.0;
        let (char_width, line_height) = get_char_dimensions(font_size);
        editor.set_char_dimensions(char_width, line_height);

        Self { editor }
    }

    fn title(&self) -> String {
        "Iced Text Editor".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Widget(widget_message) => {
                match widget_message {
                    WidgetMessage::ShortcutEvent(shortcut_event) => {
                        // Handle shortcut event using the core editor's new method
                        let _response = self.editor.handle_shortcut_event(shortcut_event);
                    }
                    WidgetMessage::MousePressed(position) => {
                        // Handle mouse click using the core editor's new method
                        let _response = self.editor.handle_mouse_click(position);
                    }
                    WidgetMessage::Scroll(delta, bounds) => {
                        // Handle scrolling using the core editor's new method
                        let _response = self.editor.handle_scroll(
                            delta.x,
                            delta.y,
                            bounds.viewport_size.width,
                            bounds.viewport_size.height,
                        );
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
    iced::run(EditorApp::update, EditorApp::view)
}
