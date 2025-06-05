use icedit::{Editor, EditorMessage, Position, UIEditor};
use icedit_core::KeyInput;

fn main() {
    // Create a new editor instance
    let mut editor = Editor::new();

    // Add an event handler to log events
    editor.add_event_handler(|event| {
        println!("Editor event: {:?}", event);
    });

    // Insert some text
    let response = editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));
    println!("Insert response: {:?}", response);

    // Move cursor to beginning
    let response = editor.handle_message(EditorMessage::MoveCursorTo(Position::new(0, 0)));
    println!("Move cursor response: {:?}", response);

    // Select all text
    let response = editor.handle_message(EditorMessage::SelectAll);
    println!("Select all response: {:?}", response);

    // Copy the text
    let response = editor.handle_message(EditorMessage::Copy);
    println!("Copy response: {:?}", response);

    // Insert a newline and paste
    let response = editor.handle_message(EditorMessage::InsertChar('\n'));
    println!("Insert newline response: {:?}", response);

    let response = editor.handle_message(EditorMessage::Paste);
    println!("Paste response: {:?}", response);

    // Get the current buffer content
    let content = editor.current_buffer().text();
    println!("Buffer content:\n{}", content);

    // Demonstrate shortcut handling with UIEditor
    let mut ui_editor = UIEditor::with_text("Hello, World!");

    // Simulate Ctrl+A key input
    let key_input = KeyInput::Command("ctrl+a".to_string());
    if let Some(response) = ui_editor.handle_key_input(key_input) {
        println!("Shortcut triggered with response: {:?}", response);
    }

    // Demonstrate undo/redo
    let response = editor.handle_message(EditorMessage::Undo);
    println!("Undo response: {:?}", response);

    let response = editor.handle_message(EditorMessage::Redo);
    println!("Redo response: {:?}", response);

    // Demonstrate editor manipulation methods
    println!("\n--- Editor Content Management ---");

    // Create editor with initial text
    let mut editor2 = Editor::with_text("Initial content\nSecond line");
    println!("Editor2 content:\n{}", editor2.current_buffer().text());

    // Clear the editor
    editor2.clear();
    println!("After clear:\n{}", editor2.current_buffer().text());

    // Set new text
    editor2.set_text("New content\nFrom set_text method");
    println!("After set_text:\n{}", editor2.current_buffer().text());

    // Show final buffer state
    println!("\n--- Final State ---");
    println!("Final buffer content:\n{}", editor.current_buffer().text());
    println!("Cursor position: {:?}", editor.current_cursor().position());
    println!("Selection: {:?}", editor.current_selection());
}
