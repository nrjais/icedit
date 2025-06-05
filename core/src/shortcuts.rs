use crate::{CursorMovement, EditorMessage, Key, KeyEvent, Modifiers, NamedKey};
use std::collections::HashMap;

/// Represents a keyboard shortcut using the new key event system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl Shortcut {
    pub fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create shortcut from key event
    pub fn from_key_event(event: KeyEvent) -> Self {
        Self {
            key: event.key,
            modifiers: event.modifiers,
        }
    }

    /// Create shortcut from string description (e.g., "ctrl+a", "shift+f3")
    pub fn from_string(desc: &str) -> Self {
        let mut modifiers = Modifiers::new();
        let parts: Vec<&str> = desc.split('+').collect();

        if let Some(key_str) = parts.last() {
            for part in &parts[..parts.len().saturating_sub(1)] {
                match part.to_lowercase().as_str() {
                    "ctrl" => modifiers.control = true,
                    "alt" => modifiers.alt = true,
                    "shift" => modifiers.shift = true,
                    "super" | "cmd" => modifiers.super_key = true,
                    _ => {}
                }
            }

            let key = match key_str.to_lowercase().as_str() {
                "left" => Key::Named(NamedKey::ArrowLeft),
                "right" => Key::Named(NamedKey::ArrowRight),
                "up" => Key::Named(NamedKey::ArrowUp),
                "down" => Key::Named(NamedKey::ArrowDown),
                "f1" => Key::Named(NamedKey::F1),
                "f2" => Key::Named(NamedKey::F2),
                "f3" => Key::Named(NamedKey::F3),
                "f4" => Key::Named(NamedKey::F4),
                "f5" => Key::Named(NamedKey::F5),
                "f6" => Key::Named(NamedKey::F6),
                "f7" => Key::Named(NamedKey::F7),
                "f8" => Key::Named(NamedKey::F8),
                "f9" => Key::Named(NamedKey::F9),
                "f10" => Key::Named(NamedKey::F10),
                "f11" => Key::Named(NamedKey::F11),
                "f12" => Key::Named(NamedKey::F12),
                "backspace" => Key::Named(NamedKey::Backspace),
                "delete" => Key::Named(NamedKey::Delete),
                "enter" => Key::Named(NamedKey::Enter),
                "escape" => Key::Named(NamedKey::Escape),
                "tab" => Key::Named(NamedKey::Tab),
                "space" => Key::Named(NamedKey::Space),
                "home" => Key::Named(NamedKey::Home),
                "end" => Key::Named(NamedKey::End),
                "pageup" => Key::Named(NamedKey::PageUp),
                "pagedown" => Key::Named(NamedKey::PageDown),
                "insert" => Key::Named(NamedKey::Insert),
                s if s.len() == 1 => Key::Character(s.chars().next().unwrap()),
                _ => Key::Character(' '), // fallback
            };

            Self::new(key, modifiers)
        } else {
            Self::new(Key::Character(' '), Modifiers::new())
        }
    }

    /// Common shortcuts
    pub fn ctrl(key: Key) -> Self {
        Self::new(key, Modifiers::new().control())
    }

    pub fn alt(key: Key) -> Self {
        Self::new(key, Modifiers::new().alt())
    }

    pub fn shift(key: Key) -> Self {
        Self::new(key, Modifiers::new().shift())
    }

    pub fn cmd(key: Key) -> Self {
        if cfg!(target_os = "macos") {
            Self::new(key, Modifiers::new().super_key())
        } else {
            Self::ctrl(key)
        }
    }

    pub fn ctrl_shift(key: Key) -> Self {
        Self::new(key, Modifiers::new().control().shift())
    }

    pub fn alt_shift(key: Key) -> Self {
        Self::new(key, Modifiers::new().alt().shift())
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

/// Events that the shortcut manager can emit
#[derive(Debug, Clone)]
pub enum ShortcutEvent {
    /// Editor message from a shortcut
    EditorMessage(EditorMessage),
    /// Character input (not a shortcut)
    CharacterInput(char),
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

    /// Handle key event - returns either a shortcut event or character input
    pub fn handle_key_event(&self, event: KeyEvent) -> Option<ShortcutEvent> {
        let shortcut = Shortcut::from_key_event(event.clone());

        // Check if this is a shortcut first
        if let Some(message) = self.bindings.get(&shortcut) {
            return Some(ShortcutEvent::EditorMessage(message.clone()));
        }

        // If not a shortcut and it's a character with no modifiers (except shift), treat as character input
        match event.key {
            Key::Character(ch) => {
                if event.modifiers.is_empty()
                    || (event.modifiers.shift
                        && !event.modifiers.control
                        && !event.modifiers.alt
                        && !event.modifiers.super_key)
                {
                    Some(ShortcutEvent::CharacterInput(ch))
                } else {
                    None
                }
            }
            Key::Named(NamedKey::Enter) => {
                if event.modifiers.is_empty() {
                    Some(ShortcutEvent::CharacterInput('\n'))
                } else {
                    None
                }
            }
            Key::Named(NamedKey::Tab) => {
                if event.modifiers.is_empty() {
                    Some(ShortcutEvent::CharacterInput('\t'))
                } else {
                    None
                }
            }
            Key::Named(NamedKey::Space) => {
                if event.modifiers.is_empty() {
                    Some(ShortcutEvent::CharacterInput(' '))
                } else {
                    None
                }
            }
            _ => None,
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
        // Basic cursor movement
        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::ArrowUp), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::Up),
            "Move cursor up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::ArrowDown), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::Down),
            "Move cursor down",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::ArrowLeft), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::Left),
            "Move cursor left",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::ArrowRight), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::Right),
            "Move cursor right",
        ));

        // Word movement
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Named(NamedKey::ArrowLeft)),
            EditorMessage::MoveCursor(CursorMovement::WordLeft),
            "Move cursor to previous word",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Named(NamedKey::ArrowRight)),
            EditorMessage::MoveCursor(CursorMovement::WordRight),
            "Move cursor to next word",
        ));

        // Line movement
        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::Home), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::LineStart),
            "Move cursor to line start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::End), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::LineEnd),
            "Move cursor to line end",
        ));

        // Document movement
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Named(NamedKey::Home)),
            EditorMessage::MoveCursor(CursorMovement::DocumentStart),
            "Move cursor to document start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Named(NamedKey::End)),
            EditorMessage::MoveCursor(CursorMovement::DocumentEnd),
            "Move cursor to document end",
        ));

        // Page movement
        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::PageUp), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::PageUp),
            "Move cursor page up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::PageDown), Modifiers::new()),
            EditorMessage::MoveCursor(CursorMovement::PageDown),
            "Move cursor page down",
        ));

        // Deletion
        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::Delete), Modifiers::new()),
            EditorMessage::DeleteChar,
            "Delete character",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::Backspace), Modifiers::new()),
            EditorMessage::DeleteCharBackward,
            "Delete character backward",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('k')),
            EditorMessage::DeleteLine,
            "Delete line",
        ));

        // Selection
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('a')),
            EditorMessage::SelectAll,
            "Select all",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('l')),
            EditorMessage::SelectLine,
            "Select line",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::Escape), Modifiers::new()),
            EditorMessage::ClearSelection,
            "Clear selection",
        ));

        // Edit operations
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('z')),
            EditorMessage::Undo,
            "Undo",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('y')),
            EditorMessage::Redo,
            "Redo",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Character('z')),
            EditorMessage::Redo,
            "Redo (alternative)",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('x')),
            EditorMessage::Cut,
            "Cut",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('c')),
            EditorMessage::Copy,
            "Copy",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('v')),
            EditorMessage::Paste,
            "Paste",
        ));

        // Search
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('f')),
            EditorMessage::Find("".to_string()),
            "Find",
        ));

        self.bind(KeyBinding::new(
            Shortcut::new(Key::Named(NamedKey::F3), Modifiers::new()),
            EditorMessage::FindNext,
            "Find next",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::F3)),
            EditorMessage::FindPrevious,
            "Find previous",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('h')),
            EditorMessage::Replace("".to_string(), "".to_string()),
            "Replace",
        ));

        // macOS specific bindings
        if cfg!(target_os = "macos") {
            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowLeft),
                    Modifiers::new().super_key(),
                ),
                EditorMessage::MoveCursor(CursorMovement::LineStart),
                "Move to line start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowRight),
                    Modifiers::new().super_key(),
                ),
                EditorMessage::MoveCursor(CursorMovement::LineEnd),
                "Move to line end (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Named(NamedKey::ArrowUp), Modifiers::new().super_key()),
                EditorMessage::MoveCursor(CursorMovement::DocumentStart),
                "Move to document start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowDown),
                    Modifiers::new().super_key(),
                ),
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
