use crate::EditorMessage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
// use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a keyboard shortcut
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Shortcut {
    pub fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create shortcut from key event
    pub fn from_key_event(event: KeyEvent) -> Self {
        Self {
            key: event.code,
            modifiers: event.modifiers,
        }
    }

    /// Common shortcuts
    pub fn ctrl(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::CONTROL)
    }

    pub fn alt(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::ALT)
    }

    pub fn shift(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::SHIFT)
    }

    pub fn cmd(key: KeyCode) -> Self {
        if cfg!(target_os = "macos") {
            Self::new(key, KeyModifiers::SUPER)
        } else {
            Self::new(key, KeyModifiers::CONTROL)
        }
    }

    pub fn ctrl_shift(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    }

    pub fn alt_shift(key: KeyCode) -> Self {
        Self::new(key, KeyModifiers::ALT | KeyModifiers::SHIFT)
    }
}

/// Represents a key binding that maps shortcuts to editor messages
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub shortcut: Shortcut,
    pub message: EditorMessage,
    pub description: String,
}

impl KeyBinding {
    pub fn new(shortcut: Shortcut, message: EditorMessage, description: &str) -> Self {
        Self {
            shortcut,
            message,
            description: description.to_string(),
        }
    }
}

/// Manages keyboard shortcuts and key bindings
#[derive(Debug, Clone)]
pub struct ShortcutManager {
    bindings: HashMap<Shortcut, EditorMessage>,
    descriptions: HashMap<Shortcut, String>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            descriptions: HashMap::new(),
        };

        manager.load_default_bindings();
        manager
    }

    /// Add a key binding
    pub fn bind(&mut self, binding: KeyBinding) {
        self.bindings
            .insert(binding.shortcut.clone(), binding.message);
        self.descriptions
            .insert(binding.shortcut, binding.description);
    }

    /// Remove a key binding
    pub fn unbind(&mut self, shortcut: &Shortcut) {
        self.bindings.remove(shortcut);
        self.descriptions.remove(shortcut);
    }

    /// Get message for a shortcut
    pub fn get_message(&self, shortcut: &Shortcut) -> Option<&EditorMessage> {
        self.bindings.get(shortcut)
    }

    /// Get message from key event
    pub fn handle_key_event(&self, event: KeyEvent) -> Option<EditorMessage> {
        let shortcut = Shortcut::from_key_event(event);
        self.bindings.get(&shortcut).cloned()
    }

    /// Get all bindings
    pub fn get_bindings(&self) -> Vec<KeyBinding> {
        self.bindings
            .iter()
            .map(|(shortcut, message)| {
                let description = self
                    .descriptions
                    .get(shortcut)
                    .cloned()
                    .unwrap_or_else(|| "No description".to_string());
                KeyBinding::new(shortcut.clone(), message.clone(), &description)
            })
            .collect()
    }

    /// Load default key bindings
    fn load_default_bindings(&mut self) {
        use crate::messages::CursorMovement;
        use KeyCode::*;

        // Basic cursor movement
        self.bind(KeyBinding::new(
            Shortcut::new(Up, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::Up),
            "Move cursor up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Down, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::Down),
            "Move cursor down",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Left, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::Left),
            "Move cursor left",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Right, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::Right),
            "Move cursor right",
        ));

        // Word movement
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Left),
            EditorMessage::MoveCursor(CursorMovement::WordLeft),
            "Move cursor to previous word",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Right),
            EditorMessage::MoveCursor(CursorMovement::WordRight),
            "Move cursor to next word",
        ));

        // Line movement
        self.bind(KeyBinding::new(
            Shortcut::new(Home, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::LineStart),
            "Move cursor to line start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(End, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::LineEnd),
            "Move cursor to line end",
        ));

        // Document movement
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Home),
            EditorMessage::MoveCursor(CursorMovement::DocumentStart),
            "Move cursor to document start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(End),
            EditorMessage::MoveCursor(CursorMovement::DocumentEnd),
            "Move cursor to document end",
        ));

        // Page movement
        self.bind(KeyBinding::new(
            Shortcut::new(PageUp, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::PageUp),
            "Move cursor page up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(PageDown, KeyModifiers::NONE),
            EditorMessage::MoveCursor(CursorMovement::PageDown),
            "Move cursor page down",
        ));

        // Deletion
        self.bind(KeyBinding::new(
            Shortcut::new(Delete, KeyModifiers::NONE),
            EditorMessage::DeleteChar,
            "Delete character",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Backspace, KeyModifiers::NONE),
            EditorMessage::DeleteCharBackward,
            "Delete character backward",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('k')),
            EditorMessage::DeleteLine,
            "Delete line",
        ));

        // Selection
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('a')),
            EditorMessage::SelectAll,
            "Select all",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('l')),
            EditorMessage::SelectLine,
            "Select line",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Esc, KeyModifiers::NONE),
            EditorMessage::ClearSelection,
            "Clear selection",
        ));

        // Edit operations
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('z')),
            EditorMessage::Undo,
            "Undo",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('y')),
            EditorMessage::Redo,
            "Redo",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Char('z')),
            EditorMessage::Redo,
            "Redo (alternative)",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('x')),
            EditorMessage::Cut,
            "Cut",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('c')),
            EditorMessage::Copy,
            "Copy",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('v')),
            EditorMessage::Paste,
            "Paste",
        ));

        // Search
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('f')),
            EditorMessage::Find("".to_string()),
            "Find",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(KeyCode::F(3), KeyModifiers::NONE),
            EditorMessage::FindNext,
            "Find next",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(KeyCode::F(3)),
            EditorMessage::FindPrevious,
            "Find previous",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Char('h')),
            EditorMessage::Replace("".to_string(), "".to_string()),
            "Replace",
        ));

        // macOS specific bindings
        if cfg!(target_os = "macos") {
            self.bind(KeyBinding::new(
                Shortcut::new(Left, KeyModifiers::SUPER),
                EditorMessage::MoveCursor(CursorMovement::LineStart),
                "Move to line start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Right, KeyModifiers::SUPER),
                EditorMessage::MoveCursor(CursorMovement::LineEnd),
                "Move to line end (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Up, KeyModifiers::SUPER),
                EditorMessage::MoveCursor(CursorMovement::DocumentStart),
                "Move to document start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Down, KeyModifiers::SUPER),
                EditorMessage::MoveCursor(CursorMovement::DocumentEnd),
                "Move to document end (macOS)",
            ));
        }
    }

    /// Load bindings from configuration
    pub fn load_from_config(&mut self, bindings: Vec<KeyBinding>) {
        for binding in bindings {
            self.bind(binding);
        }
    }

    /// Export bindings to configuration format
    pub fn export_config(&self) -> Vec<KeyBinding> {
        self.get_bindings()
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}
