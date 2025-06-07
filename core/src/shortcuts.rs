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
    pub fn handle_key_event(&self, event: KeyEvent) -> Option<EditorMessage> {
        let shortcut = Shortcut::from_key_event(event.clone());

        // Check if this is a shortcut first
        if let Some(message) = self.bindings.get(&shortcut) {
            return Some(message.clone());
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
                    Some(EditorMessage::InsertChar(ch))
                } else {
                    None
                }
            }
            Key::Named(NamedKey::Enter) => {
                if event.modifiers.is_empty() {
                    Some(EditorMessage::InsertChar('\n'))
                } else {
                    None
                }
            }
            Key::Named(NamedKey::Tab) => {
                if event.modifiers.is_empty() {
                    Some(EditorMessage::InsertChar('\t'))
                } else {
                    None
                }
            }
            Key::Named(NamedKey::Space) => {
                if event.modifiers.is_empty() {
                    Some(EditorMessage::InsertChar(' '))
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

        // Selection with Shift
        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::ArrowUp)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::Up),
            "Select up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::ArrowDown)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::Down),
            "Select down",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::ArrowLeft)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::Left),
            "Select left",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::ArrowRight)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::Right),
            "Select right",
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

        // Word selection
        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Named(NamedKey::ArrowLeft)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::WordLeft),
            "Select to previous word",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Named(NamedKey::ArrowRight)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::WordRight),
            "Select to next word",
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

        // Line selection
        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::Home)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::LineStart),
            "Select to line start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::End)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::LineEnd),
            "Select to line end",
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

        // Document selection
        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Named(NamedKey::Home)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::DocumentStart),
            "Select to document start",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Named(NamedKey::End)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::DocumentEnd),
            "Select to document end",
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

        // Page selection
        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::PageUp)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::PageUp),
            "Select page up",
        ));

        self.bind(KeyBinding::new(
            Shortcut::shift(Key::Named(NamedKey::PageDown)),
            EditorMessage::MoveCursorWithSelection(CursorMovement::PageDown),
            "Select page down",
        ));

        // Basic deletion
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

        // Word deletion
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Named(NamedKey::Delete)),
            EditorMessage::DeleteWordForward,
            "Delete word forward",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Named(NamedKey::Backspace)),
            EditorMessage::DeleteWordBackward,
            "Delete word backward",
        ));

        // Advanced line deletion
        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Named(NamedKey::Delete)),
            EditorMessage::DeleteToLineEnd,
            "Delete to end of line",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Named(NamedKey::Backspace)),
            EditorMessage::DeleteToLineStart,
            "Delete to start of line",
        ));

        // Alternative shortcuts for delete to line end (common in many editors)
        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('k')),
            EditorMessage::DeleteToLineEnd,
            "Delete to end of line (alternative)",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl(Key::Character('u')),
            EditorMessage::DeleteToLineStart,
            "Delete to start of line (alternative)",
        ));

        // Line operations (delete entire line)
        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Character('k')),
            EditorMessage::DeleteLine,
            "Delete entire line",
        ));

        self.bind(KeyBinding::new(
            Shortcut::ctrl_shift(Key::Character('l')),
            EditorMessage::DeleteLine,
            "Delete entire line (alternative)",
        ));

        // Selection operations
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
            Shortcut::ctrl(Key::Character('w')),
            EditorMessage::SelectWord,
            "Select word",
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

        // Clipboard operations
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

        // Search and replace
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
            // Basic movement with Cmd key (Super)
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

            // Selection with Cmd+Shift
            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowLeft),
                    Modifiers::new().super_key().shift(),
                ),
                EditorMessage::MoveCursorWithSelection(CursorMovement::LineStart),
                "Select to line start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowRight),
                    Modifiers::new().super_key().shift(),
                ),
                EditorMessage::MoveCursorWithSelection(CursorMovement::LineEnd),
                "Select to line end (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowUp),
                    Modifiers::new().super_key().shift(),
                ),
                EditorMessage::MoveCursorWithSelection(CursorMovement::DocumentStart),
                "Select to document start (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::ArrowDown),
                    Modifiers::new().super_key().shift(),
                ),
                EditorMessage::MoveCursorWithSelection(CursorMovement::DocumentEnd),
                "Select to document end (macOS)",
            ));

            // macOS specific clipboard shortcuts (Cmd instead of Ctrl)
            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('a'), Modifiers::new().super_key()),
                EditorMessage::SelectAll,
                "Select all (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('x'), Modifiers::new().super_key()),
                EditorMessage::Cut,
                "Cut (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('c'), Modifiers::new().super_key()),
                EditorMessage::Copy,
                "Copy (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('v'), Modifiers::new().super_key()),
                EditorMessage::Paste,
                "Paste (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('z'), Modifiers::new().super_key()),
                EditorMessage::Undo,
                "Undo (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('z'), Modifiers::new().super_key().shift()),
                EditorMessage::Redo,
                "Redo (macOS)",
            ));

            // macOS specific word movement with Option (Alt)
            self.bind(KeyBinding::new(
                Shortcut::alt(Key::Named(NamedKey::ArrowLeft)),
                EditorMessage::MoveCursor(CursorMovement::WordLeft),
                "Move to previous word (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::alt(Key::Named(NamedKey::ArrowRight)),
                EditorMessage::MoveCursor(CursorMovement::WordRight),
                "Move to next word (macOS)",
            ));

            // macOS word selection with Option+Shift
            self.bind(KeyBinding::new(
                Shortcut::alt_shift(Key::Named(NamedKey::ArrowLeft)),
                EditorMessage::MoveCursorWithSelection(CursorMovement::WordLeft),
                "Select to previous word (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::alt_shift(Key::Named(NamedKey::ArrowRight)),
                EditorMessage::MoveCursorWithSelection(CursorMovement::WordRight),
                "Select to next word (macOS)",
            ));

            // macOS word deletion with Option
            self.bind(KeyBinding::new(
                Shortcut::alt(Key::Named(NamedKey::Delete)),
                EditorMessage::DeleteWordForward,
                "Delete word forward (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::alt(Key::Named(NamedKey::Backspace)),
                EditorMessage::DeleteWordBackward,
                "Delete word backward (macOS)",
            ));

            // macOS advanced deletion shortcuts
            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('k'), Modifiers::new().super_key()),
                EditorMessage::DeleteToLineEnd,
                "Delete to end of line (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('u'), Modifiers::new().super_key()),
                EditorMessage::DeleteToLineStart,
                "Delete to start of line (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Named(NamedKey::Delete), Modifiers::new().super_key()),
                EditorMessage::DeleteToLineEnd,
                "Delete to end of line with Cmd+Delete (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(
                    Key::Named(NamedKey::Backspace),
                    Modifiers::new().super_key(),
                ),
                EditorMessage::DeleteToLineStart,
                "Delete to start of line with Cmd+Backspace (macOS)",
            ));

            // macOS specific search shortcuts
            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('f'), Modifiers::new().super_key()),
                EditorMessage::Find("".to_string()),
                "Find (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('g'), Modifiers::new().super_key()),
                EditorMessage::FindNext,
                "Find next (macOS)",
            ));

            self.bind(KeyBinding::new(
                Shortcut::new(Key::Character('g'), Modifiers::new().super_key().shift()),
                EditorMessage::FindPrevious,
                "Find previous (macOS)",
            ));
        }

        // Windows/Linux specific shortcuts (beyond the defaults)
        if !cfg!(target_os = "macos") {
            // Additional Windows/Linux specific shortcuts can be added here
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
