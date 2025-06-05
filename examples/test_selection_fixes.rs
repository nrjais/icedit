use iced::{Element, Task};
use icedit_core::{Editor, EditorMessage, ShortcutEvent};
use icedit_ui::{styled_editor, WidgetMessage};

/// Test application to verify all selection fixes
struct SelectionFixTestApp {
    editor: Editor,
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    Widget(WidgetMessage),
}

impl SelectionFixTestApp {
    fn new() -> Self {
        // Create test text specifically designed to test various selection issues
        let test_text = concat!(
            "Line 1: Short line\n",
            "Line 2: This is a longer line with more characters to test end-of-line selection\n",
            "Line 3: Medium line\n",
            "Line 4: Another line\n",
            "Line 5: Test line with tabs\tand\tspaces\n",
            "Line 6: Final line without newline"
        );

        let mut editor = Editor::with_text(test_text);

        // Initialize viewport with reasonable defaults
        editor.set_viewport_size(800.0, 600.0);
        editor.set_char_dimensions(8.0, 18.0);

        Self { editor }
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
}

impl Default for SelectionFixTestApp {
    fn default() -> Self {
        Self::new()
    }
}

fn main() -> iced::Result {
    println!("Selection Fixes Test");
    println!("====================");
    println!("This example tests the following fixes:");
    println!("1. Last character selection (Shift+End, Shift+Arrow keys)");
    println!("2. Line selection (Ctrl+L or Cmd+L)");
    println!("3. Document selection (Ctrl+A or Cmd+A)");
    println!("4. Selection highlighting without overlaps");
    println!();
    println!("Test Instructions:");
    println!("- Use Shift+End to select to end of line");
    println!("- Use Shift+Arrow keys to extend selection");
    println!("- Use Ctrl+L (Cmd+L) to select entire lines");
    println!("- Use Ctrl+A (Cmd+A) to select all text");
    println!("- Observe that selection highlights don't overlap");

    iced::run(
        "Selection Fixes Test",
        SelectionFixTestApp::update,
        SelectionFixTestApp::view,
    )
}
