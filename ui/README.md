# IcEdit UI - Iced Widget

This crate provides an Iced widget for the IcEdit text editor that renders text using custom drawing with Canvas-like rendering.

## Features

- **External State Management**: The widget state is managed externally, allowing for flexible integration
- **Message Routing**: All widget messages are routed through user-defined message types
- **Canvas Rendering**: Text is rendered using Iced's advanced rendering capabilities for optimal performance
- **Customizable Appearance**: Font size, colors, and themes can be customized
- **Full Editor Functionality**: Supports all core editor features including cursor movement, selection, copy/paste, undo/redo

## Usage

### Basic Setup

```rust
use iced::{Element, Task, Theme};
use icedit_core::Editor;
use icedit_ui::{styled_editor, EditorState, WidgetMessage};

// Define your application message type
#[derive(Debug, Clone)]
enum Message {
    Widget(WidgetMessage),
}

// Create your application state
struct App {
    editor: Editor,
    editor_state: EditorState,
}

impl App {
    fn new() -> Self {
        let editor = Editor::with_text("Hello, World!");
        let editor_state = EditorState::from_editor(&editor);

        Self { editor, editor_state }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Widget(widget_message) => {
                match widget_message {
                    WidgetMessage::Editor(editor_message) => {
                        // Handle editor messages
                        self.editor.handle_message(editor_message);
                        self.update_editor_state();
                    }
                    WidgetMessage::Scroll(delta) => {
                        // Handle scrolling
                        self.editor_state.scroll_offset += delta;
                    }
                    _ => {}
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        styled_editor(
            self.editor_state.clone(),
            16.0,            // Font size
            true,            // Dark theme
            Message::Widget, // Message mapper
        )
    }

    fn update_editor_state(&mut self) {
        let old_scroll = self.editor_state.scroll_offset;
        self.editor_state = EditorState::from_editor(&self.editor);
        self.editor_state.scroll_offset = old_scroll;
    }
}
```

### Widget State

The `EditorState` struct contains all the visual state needed by the widget:

```rust
pub struct EditorState {
    pub buffer_content: String,      // The text content
    pub cursor_position: Position,   // Current cursor position
    pub selection: Option<Selection>, // Current selection (if any)
    pub scroll_offset: Vector,       // Scroll position
}
```

### Widget Messages

The widget emits `WidgetMessage` enum variants:

```rust
pub enum WidgetMessage {
    Editor(EditorMessage),    // Core editor operations
    Scroll(Vector),          // Scroll events
    MousePressed(Point),     // Mouse interactions
    MouseReleased(Point),
    MouseMoved(Point),
}
```

### Customization

You can customize the widget appearance:

```rust
use iced::Color;

// Custom colors
let widget = styled_editor(
    state,
    18.0,  // Larger font
    false, // Light theme
    Message::Widget,
);

// Or use the basic widget with manual styling
let widget = editor_widget(state, Message::Widget)
    .font_size(20.0)
    .colors(
        Color::WHITE,                        // Background
        Color::BLACK,                        // Text
        Color::RED,                          // Cursor
        Color::from_rgba(0.0, 0.0, 1.0, 0.3), // Selection
    );
```

## Key Features

### Keyboard Support

The widget handles all standard text editing keyboard shortcuts:

- **Text Input**: Regular typing, Enter for new lines
- **Navigation**: Arrow keys, Home/End, Page Up/Down
- **Selection**: Shift + navigation keys
- **Editing**: Backspace, Delete
- **Clipboard**: Ctrl+C (copy), Ctrl+V (paste), Ctrl+X (cut)
- **Undo/Redo**: Ctrl+Z (undo), Ctrl+Y (redo)
- **Select All**: Ctrl+A

### Mouse Support

- **Click to Position**: Click anywhere to move the cursor
- **Scroll**: Mouse wheel scrolling (both vertical and horizontal)
- **Selection**: Click and drag to select text (planned feature)

### Performance

The widget is optimized for performance:

- Only visible lines are rendered
- Efficient text measurement and positioning
- Minimal redraws when possible
- Scroll-aware rendering

## Architecture

The widget follows a clean separation of concerns:

1. **Core Editor** (`icedit-core`): Handles all text editing logic
2. **Widget State** (`EditorState`): Contains visual state for rendering
3. **Widget Messages** (`WidgetMessage`): Communication between widget and application
4. **Rendering**: Custom drawing using Iced's advanced renderer

This architecture allows for:

- **Testable Logic**: Core editor logic is separate from UI
- **Flexible Integration**: Widget can be used in any Iced application
- **State Management**: Application controls when and how state updates
- **Message Routing**: Application decides how to handle widget messages

## Example

See `examples/iced_editor.rs` for a complete working example:

```bash
cargo run --example iced_editor
```

This example demonstrates:

- Setting up the widget in an Iced application
- Handling widget messages
- Managing editor state
- Implementing scrolling
- Using the dark theme
