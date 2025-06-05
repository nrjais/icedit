# IcEdit - Headless Code Editor

A headless code editor built on top of the ropey Rust library with a message-based architecture and cross-platform shortcut system.

## Features

- **Headless Architecture**: Core editor logic separated from UI, allowing for multiple frontend implementations
- **Message-Based Design**: All editor actions are represented as messages for easy integration and testing
- **Cross-Platform Shortcuts**: Native keybinding support for Windows, macOS, and Linux
- **Ropey Integration**: Efficient text manipulation using the ropey rope data structure
- **Undo/Redo System**: Built-in undo/redo functionality with configurable history levels
- **Search and Replace**: Text search and replacement functionality
- **Event System**: Extensible event system for UI integration

## Architecture

The editor is built around several core components:

### Core Components

- **Editor**: Main editor state and message handler
- **Buffer**: Text buffer wrapper around ropey with undo/redo
- **Cursor**: Cursor position and movement logic
- **Selection**: Text selection handling
- **Messages**: All possible editor actions as enum variants
- **Shortcuts**: Cross-platform keyboard shortcut management

### Message-Based Design

All editor operations are performed by sending messages:

```rust
use icedit::{Editor, EditorMessage, Position};

let mut editor = Editor::new();

// Insert text
editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));

// Move cursor
editor.handle_message(EditorMessage::MoveCursorTo(Position::new(0, 5)));

// Select all
editor.handle_message(EditorMessage::SelectAll);

// Copy and paste
editor.handle_message(EditorMessage::Copy);
editor.handle_message(EditorMessage::Paste);
```

### Available Messages

#### Text Manipulation
- `InsertChar(char)` - Insert a single character
- `InsertText(String)` - Insert text string
- `DeleteChar` - Delete character at cursor
- `DeleteCharBackward` - Delete character before cursor (backspace)
- `DeleteLine` - Delete entire line
- `DeleteSelection` - Delete selected text

#### Cursor Movement
- `MoveCursor(CursorMovement)` - Move cursor (Up, Down, Left, Right, etc.)
- `MoveCursorTo(Position)` - Move cursor to specific position

#### Selection
- `StartSelection` - Start text selection
- `EndSelection` - End text selection
- `SelectAll` - Select all text
- `SelectLine` - Select current line
- `SelectWord` - Select word at cursor
- `ClearSelection` - Clear current selection

#### Edit Operations
- `Undo` - Undo last operation
- `Redo` - Redo last undone operation
- `Cut` - Cut selected text
- `Copy` - Copy selected text
- `Paste` - Paste from clipboard

#### Search and Replace
- `Find(String)` - Find text pattern
- `FindNext` - Find next occurrence
- `FindPrevious` - Find previous occurrence
- `Replace(String, String)` - Replace text
- `ReplaceAll(String, String)` - Replace all occurrences

### Shortcut System

The editor includes a comprehensive shortcut system with platform-specific bindings:

```rust
use icedit::{ShortcutManager, Shortcut, KeyBinding, EditorMessage};
use crossterm::event::{KeyCode, KeyModifiers};

let mut shortcuts = ShortcutManager::new();

// Add custom binding
let binding = KeyBinding::new(
    Shortcut::ctrl(KeyCode::Char('d')),
    EditorMessage::DeleteLine,
    "Delete current line"
);
shortcuts.bind(binding);

// Handle key events
let key_event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
if let Some(message) = shortcuts.handle_key_event(key_event) {
    let response = editor.handle_message(message);
}
```

#### Default Shortcuts

**Basic Movement:**
- Arrow keys: Move cursor
- Ctrl+Left/Right: Word movement
- Home/End: Line start/end
- Ctrl+Home/End: Document start/end
- Page Up/Down: Page movement

**Text Operations:**
- Delete/Backspace: Character deletion
- Ctrl+K: Delete line
- Ctrl+A: Select all
- Ctrl+L: Select line

**Edit Operations:**
- Ctrl+Z: Undo
- Ctrl+Y: Redo
- Ctrl+X: Cut
- Ctrl+C: Copy
- Ctrl+V: Paste

**Search:**
- Ctrl+F: Find
- F3: Find next
- Shift+F3: Find previous
- Ctrl+H: Replace

**macOS Specific:**
- Cmd+Left/Right: Line start/end
- Cmd+Up/Down: Document start/end

### Event System

The editor emits events that can be handled by UI layers:

```rust
editor.add_event_handler(|event| {
    match event {
        EditorEvent::TextChanged => println!("Text was modified"),
        EditorEvent::CursorMoved(pos) => println!("Cursor moved to {:?}", pos),
        EditorEvent::SelectionChanged(sel) => println!("Selection: {:?}", sel),
        _ => {}
    }
});
```

## Usage

### Basic Text Editing

```rust
use icedit::{Editor, EditorMessage};

let mut editor = Editor::new();

// Insert some text
editor.handle_message(EditorMessage::InsertText("fn main() {\n    println!(\"Hello, World!\");\n}".to_string()));

// Get buffer content
let content = editor.current_buffer().text();
println!("{}", content);
```

### Creating Editor with Initial Content

```rust
// Create editor with initial text
let mut editor = Editor::with_text("Initial content\nSecond line");

// Clear the editor
editor.clear();

// Set new content
editor.set_text("New content here");
```

### Content Management

```rust
// Create new editor
let mut editor = Editor::new();

// Insert text
editor.handle_message(EditorMessage::InsertText("Hello, World!".to_string()));

// Clear all content
editor.clear();

// Set specific content
editor.set_text("fn main() {\n    println!(\"Hello, Rust!\");\n}");
```

## Building UI Layers

The headless design makes it easy to build different UI layers:

### Terminal UI
Use libraries like `crossterm` or `termion` to create a terminal-based interface.

### GUI
Use frameworks like `egui`, `iced`, or `tauri` for desktop applications.

### Web
Use `wasm-bindgen` to compile to WebAssembly for web-based editors.

### Example Integration

```rust
// Terminal UI example
use crossterm::event::{read, Event, KeyEvent};

loop {
    if let Ok(Event::Key(key_event)) = read() {
        if let Some(message) = editor.shortcut_manager().handle_key_event(key_event) {
            editor.handle_message(message);
        } else if let KeyEvent { code: KeyCode::Char(c), .. } = key_event {
            editor.handle_message(EditorMessage::InsertChar(c));
        }
    }
}
```

## Dependencies

- `ropey`: Efficient rope data structure for text
- `serde`: Serialization support for messages
- `thiserror`: Error handling
- `crossterm`: Cross-platform terminal handling
- `unicode-width`: Unicode text width calculations

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Roadmap

- [ ] Syntax highlighting support
- [ ] Language server protocol integration
- [ ] Plugin system
- [ ] Configuration system
- [ ] Advanced search (regex, case sensitivity)
- [ ] Multiple cursors
- [ ] Collaborative editing
- [ ] Performance optimizations
