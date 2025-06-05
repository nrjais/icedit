use iced::{Element, Task, Theme};
use icedit_core::{Editor, EditorMessage, KeyInput};
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
                    WidgetMessage::KeyInput(key_input) => {
                        // Handle key input using the core editor's simplified interface
                        let _response = self.editor.handle_key_input(key_input);
                        self.update_editor_state();
                    }
                    WidgetMessage::MousePressed(point) => {
                        // Convert mouse position to cursor position
                        let position = self.point_to_position(point);
                        let _response = self
                            .editor
                            .handle_message(EditorMessage::MoveCursorTo(position));
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

    /// Convert screen point to editor position (simplified)
    fn point_to_position(&self, point: iced::Point) -> icedit_core::Position {
        let line_height = 18.0; // Should match widget's line height
        let char_width = 8.0; // Should match widget's char width

        let line = ((point.y + self.editor_state.scroll_offset.y) / line_height).max(0.0) as usize;
        let column = ((point.x + self.editor_state.scroll_offset.x) / char_width).max(0.0) as usize;

        // Clamp to actual text bounds (simplified)
        let lines: Vec<&str> = self.editor_state.buffer_content.lines().collect();
        let line = line.min(lines.len().saturating_sub(1));
        let column = if line < lines.len() {
            column.min(lines[line].len())
        } else {
            0
        };

        icedit_core::Position::new(line, column)
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
