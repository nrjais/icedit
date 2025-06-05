/// Represents a keyboard key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    /// Character keys
    Character(char),
    /// Named keys
    Named(NamedKey),
}

/// Named keys that aren't characters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NamedKey {
    // Arrow keys
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Special keys
    Backspace,
    Delete,
    Enter,
    Escape,
    Tab,
    Space,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,

    // Modifier keys
    Shift,
    Control,
    Alt,
    Super,
    Meta,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn control(mut self) -> Self {
        self.control = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn super_key(mut self) -> Self {
        self.super_key = true;
        self
    }

    /// Check if any modifiers are pressed
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.control && !self.alt && !self.super_key
    }
}

/// A key event with modifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    pub fn new(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Create a key event for a character
    pub fn character(ch: char) -> Self {
        Self::new(Key::Character(ch), Modifiers::new())
    }

    /// Create a key event for a named key
    pub fn named(key: NamedKey) -> Self {
        Self::new(Key::Named(key), Modifiers::new())
    }

    /// Create a key event with modifiers
    pub fn with_modifiers(key: Key, modifiers: Modifiers) -> Self {
        Self::new(key, modifiers)
    }
}

/// Convert key events to string representations for compatibility
impl KeyEvent {
    pub fn to_string_repr(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.control {
            parts.push("ctrl");
        }
        if self.modifiers.alt {
            parts.push("alt");
        }
        if self.modifiers.shift {
            parts.push("shift");
        }
        if self.modifiers.super_key {
            if cfg!(target_os = "macos") {
                parts.push("cmd");
            } else {
                parts.push("super");
            }
        }

        let key_str = match &self.key {
            Key::Character(ch) => ch.to_string(),
            Key::Named(named) => match named {
                NamedKey::ArrowLeft => "left".to_string(),
                NamedKey::ArrowRight => "right".to_string(),
                NamedKey::ArrowUp => "up".to_string(),
                NamedKey::ArrowDown => "down".to_string(),
                NamedKey::F1 => "f1".to_string(),
                NamedKey::F2 => "f2".to_string(),
                NamedKey::F3 => "f3".to_string(),
                NamedKey::F4 => "f4".to_string(),
                NamedKey::F5 => "f5".to_string(),
                NamedKey::F6 => "f6".to_string(),
                NamedKey::F7 => "f7".to_string(),
                NamedKey::F8 => "f8".to_string(),
                NamedKey::F9 => "f9".to_string(),
                NamedKey::F10 => "f10".to_string(),
                NamedKey::F11 => "f11".to_string(),
                NamedKey::F12 => "f12".to_string(),
                NamedKey::Backspace => "backspace".to_string(),
                NamedKey::Delete => "delete".to_string(),
                NamedKey::Enter => "enter".to_string(),
                NamedKey::Escape => "escape".to_string(),
                NamedKey::Tab => "tab".to_string(),
                NamedKey::Space => "space".to_string(),
                NamedKey::Home => "home".to_string(),
                NamedKey::End => "end".to_string(),
                NamedKey::PageUp => "pageup".to_string(),
                NamedKey::PageDown => "pagedown".to_string(),
                NamedKey::Insert => "insert".to_string(),
                NamedKey::Shift => "shift".to_string(),
                NamedKey::Control => "control".to_string(),
                NamedKey::Alt => "alt".to_string(),
                NamedKey::Super => "super".to_string(),
                NamedKey::Meta => "meta".to_string(),
            },
        };

        parts.push(&key_str);
        parts.join("+")
    }
}
