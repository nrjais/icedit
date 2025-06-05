// Re-export both core and ui crates for convenience
pub use icedit_core as core;
pub use icedit_ui as ui;

// Re-export commonly used types from both crates for backward compatibility
pub use icedit_core::{
    Buffer, Cursor, CursorMovement, Editor, EditorEvent, EditorMessage, EditorResponse, Position,
    Selection,
};

pub use icedit_ui::{KeyBinding, Shortcut, ShortcutManager};
