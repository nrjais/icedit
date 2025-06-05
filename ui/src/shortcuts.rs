use icedit_core::{EditorMessage, KeyInput};
use std::collections::HashMap;

/// Represents a keyboard shortcut using string-based keys for simplicity
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

impl Shortcut {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
        }
    }

    /// Create shortcut from string description (e.g., "ctrl+a", "shift+f3")
    pub fn from_string(desc: &str) -> Self {
        let mut shortcut = Self::new("");
        let parts: Vec<&str> = desc.split('+').collect();

        if let Some(key) = parts.last() {
            shortcut.key = key.to_string();
        }

        for part in &parts[..parts.len().saturating_sub(1)] {
            match part.to_lowercase().as_str() {
                "ctrl" => shortcut.ctrl = true,
                "alt" => shortcut.alt = true,
                "shift" => shortcut.shift = true,
                "super" | "cmd" => shortcut.super_key = true,
                _ => {}
            }
        }

        shortcut
    }

    /// Common shortcuts
    pub fn ctrl(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: true,
            alt: false,
            shift: false,
            super_key: false,
        }
    }

    pub fn alt(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            alt: true,
            shift: false,
            super_key: false,
        }
    }

    pub fn shift(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            alt: false,
            shift: true,
            super_key: false,
        }
    }

    pub fn cmd(key: &str) -> Self {
        if cfg!(target_os = "macos") {
            Self {
                key: key.to_string(),
                ctrl: false,
                alt: false,
                shift: false,
                super_key: true,
            }
        } else {
            Self::ctrl(key)
        }
    }

    pub fn ctrl_shift(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: true,
            alt: false,
            shift: true,
            super_key: false,
        }
    }

    pub fn alt_shift(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            alt: true,
            shift: true,
            super_key: false,
        }
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

    /// Get message from KeyInput - converts simple commands to editor messages
    pub fn handle_key_input(&self, input: &KeyInput) -> Option<EditorMessage> {
        match input {
            KeyInput::Command(cmd) => {
                let shortcut = Shortcut::from_string(cmd);
                self.bindings.get(&shortcut).cloned()
            }
            KeyInput::Character(_) => None, // Character input is handled directly
        }
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
        use icedit_core::CursorMovement;

        // Basic cursor movement
        self.bind(KeyBinding::new(
            Shortcut::new("up"),
            EditorMessage::MoveCursor(CursorMovement::Up),
            "Move cursor up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("down"),
            EditorMessage::MoveCursor(CursorMovement::Down),
            "Move cursor down",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("left"),
            EditorMessage::MoveCursor(CursorMovement::Left),
            "Move cursor left",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("right"),
            EditorMessage::MoveCursor(CursorMovement::Right),
            "Move cursor right",
        ));

        // Word movement
        self.bind(KeyBinding::new(
            Shortcut::ctrl("left"),
            EditorMessage::MoveCursor(CursorMovement::WordLeft),
            "Move cursor to previous word",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("right"),
            EditorMessage::MoveCursor(CursorMovement::WordRight),
            "Move cursor to next word",
        ));

        // Line movement
        self.bind(KeyBinding::new(
            Shortcut::new("home"),
            EditorMessage::MoveCursor(CursorMovement::LineStart),
            "Move cursor to line start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("end"),
            EditorMessage::MoveCursor(CursorMovement::LineEnd),
            "Move cursor to line end",
        ));

        // Document movement
        self.bind(KeyBinding::new(
            Shortcut::ctrl("home"),
            EditorMessage::MoveCursor(CursorMovement::DocumentStart),
            "Move cursor to document start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("end"),
            EditorMessage::MoveCursor(CursorMovement::DocumentEnd),
            "Move cursor to document end",
        ));

        // Page movement
        self.bind(KeyBinding::new(
            Shortcut::new("pageup"),
            EditorMessage::MoveCursor(CursorMovement::PageUp),
            "Move cursor page up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("pagedown"),
            EditorMessage::MoveCursor(CursorMovement::PageDown),
            "Move cursor page down",
        ));

        // Deletion
        self.bind(KeyBinding::new(
            Shortcut::new("delete"),
            EditorMessage::DeleteChar,
            "Delete character",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("backspace"),
            EditorMessage::DeleteCharBackward,
            "Delete character backward",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("k"),
            EditorMessage::DeleteLine,
            "Delete line",
        ));

        // Selection
        self.bind(KeyBinding::new(
            Shortcut::ctrl("a"),
            EditorMessage::SelectAll,
            "Select all",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("l"),
            EditorMessage::SelectLine,
            "Select line",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("escape"),
            EditorMessage::ClearSelection,
            "Clear selection",
        ));

        // Edit operations
        self.bind(KeyBinding::new(
            Shortcut::ctrl("z"),
            EditorMessage::Undo,
            "Undo",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("y"),
            EditorMessage::Redo,
            "Redo",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift("z"),
            EditorMessage::Redo,
            "Redo (alternative)",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("x"),
            EditorMessage::Cut,
            "Cut",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("c"),
            EditorMessage::Copy,
            "Copy",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("v"),
            EditorMessage::Paste,
            "Paste",
        ));

        // Search
        self.bind(KeyBinding::new(
            Shortcut::ctrl("f"),
            EditorMessage::Find("".to_string()),
            "Find",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new("f3"),
            EditorMessage::FindNext,
            "Find next",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift("f3"),
            EditorMessage::FindPrevious,
            "Find previous",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl("h"),
            EditorMessage::Replace("".to_string(), "".to_string()),
            "Replace",
        ));

        // macOS specific bindings
        if cfg!(target_os = "macos") {
            self.bind(KeyBinding::new(
                Shortcut::from_string("super+left"),
                EditorMessage::MoveCursor(CursorMovement::LineStart),
                "Move to line start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::from_string("super+right"),
                EditorMessage::MoveCursor(CursorMovement::LineEnd),
                "Move to line end (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::from_string("super+up"),
                EditorMessage::MoveCursor(CursorMovement::DocumentStart),
                "Move to document start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::from_string("super+down"),
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
