use std::fs;

use iced::{Element, Task, Theme};
use icedit::EditorMessage;
use icedit_core::Editor;
use icedit_ui::styled_editor;

/// Main application state
struct EditorApp {
    editor: Editor,
}

/// Application messages
#[derive(Debug, Clone)]
enum Message {
    /// Widget messages from the editor
    Editor(EditorMessage),
}

impl EditorApp {
    fn new() -> (Self, Task<Message>) {
        // Try to read README.md, but fall back to demo content with tabs if it fails
        let text = fs::read_to_string("README.md").unwrap_or_else(|_| {
            "# Ice Edit - Tab Handling Demo\n\n\
            This editor properly handles tabs for:\n\n\
            \t• Cursor positioning\n\
            \t• Text selection\n\
            \t• Horizontal scrolling\n\
            \t• Rendering alignment\n\n\
            Multiple tab testing:\n\
            \t\tDouble tabs at start\n\
            hello\t\tworld with double tabs\n\
            \t\ta\tb\tc\tmixed content\n\n\
            function example() {\n\
            \tif (condition) {\n\
            \t\treturn value;\n\
            \t}\n\
            }\n\n\
            Edge cases:\n\
            \t\t\t\tFour tabs\n\
            a\t\t\t\tb\n\
            Try clicking at different positions in lines with multiple tabs!"
                .to_string()
        });
        let editor = Editor::with_text(&text);

        let app = Self { editor };

        (app, Task::none())
    }

    fn title(&self) -> String {
        "Iced Text Editor".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Editor(editor_message) => {
                self.editor.handle_message(editor_message);
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
            Message::Editor, // Message mapper
        )
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn main() -> iced::Result {
    iced::application(EditorApp::new, EditorApp::update, EditorApp::view)
        .title(|app: &EditorApp| app.title())
        .theme(|app: &EditorApp| app.theme())
        .run()
}
