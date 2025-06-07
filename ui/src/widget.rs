use crate::renderer::EditorRenderer;
use iced::{
    advanced::{
        layout::{self, Layout},
        renderer::{self},
        widget::Tree,
        Clipboard, Shell, Widget,
    },
    mouse, Color, Element, Event, Font, Length, Point, Rectangle, Size, Theme, Vector,
};
use icedit_core::{
    Editor, EditorMessage, Key, KeyEvent, Modifiers, NamedKey, Position, Selection, ShortcutManager,
};

/// State that should be passed from outside to the widget
#[derive(Debug, Clone)]
pub struct EditorState {
    pub cursor_position: Position,
    pub selection: Option<Selection>,
    pub scroll_offset: Vector,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            cursor_position: Position::zero(),
            selection: None,
            scroll_offset: Vector::ZERO,
        }
    }

    pub fn from_editor(editor: &Editor) -> Self {
        let viewport = editor.viewport();
        Self {
            cursor_position: editor.current_cursor().position(),
            selection: editor.current_selection().cloned(),
            scroll_offset: Vector::new(viewport.scroll_offset.0, viewport.scroll_offset.1),
        }
    }
}

/// Messages that the widget can emit - these will be routed by the user
#[derive(Debug, Clone)]
pub enum WidgetMessage {
    EditorMessage(EditorMessage),
    /// Scroll events with viewport bounds for proper scroll limiting
    Scroll(Vector, ScrollBounds),
    /// Mouse events with editor positions
    MousePressed(Position),
    MouseReleased(Position),
    MouseMoved(Position),
}

/// Information about scrolling bounds
#[derive(Debug, Clone)]
pub struct ScrollBounds {
    /// Content size (width, height)
    pub content_size: Size,
    /// Viewport size (width, height)
    pub viewport_size: Size,
    /// Line height for text
    pub line_height: f32,
}

impl ScrollBounds {
    /// Calculate proper scroll offset with bounds checking
    pub fn clamp_scroll_offset(&self, offset: Vector) -> Vector {
        let mut new_offset = offset;

        // Clamp to minimum bounds (top-left)
        new_offset.y = new_offset.y.max(0.0);
        new_offset.x = new_offset.x.max(0.0);

        // Clamp to maximum bounds (bottom-right)
        if self.content_size.height > self.viewport_size.height {
            let max_scroll = self.content_size.height - self.viewport_size.height;
            new_offset.y = new_offset.y.min(max_scroll);
        } else {
            new_offset.y = 0.0;
        }

        if self.content_size.width > self.viewport_size.width {
            let max_scroll = self.content_size.width - self.viewport_size.width;
            new_offset.x = new_offset.x.min(max_scroll);
        } else {
            new_offset.x = 0.0;
        }

        new_offset
    }
}

/// The editor widget that renders text using custom drawing
pub struct EditorWidget<'a, Message> {
    editor: &'a Editor,
    font_size: f32,
    line_height: f32,
    char_width: f32,
    background_color: Color,
    text_color: Color,
    cursor_color: Color,
    selection_color: Color,
    shortcut_manager: ShortcutManager,
    on_message: Box<dyn Fn(EditorMessage) -> Message>,
}

impl<'a, Message> EditorWidget<'a, Message> {
    const DEFAULT_FONT_SIZE: f32 = 14.0;

    pub fn new<F>(editor: &'a Editor, on_message: F) -> Self
    where
        F: Fn(EditorMessage) -> Message + 'static,
    {
        let (char_width, line_height) = Self::measure_char_dimensions(Self::DEFAULT_FONT_SIZE);

        let widget = Self {
            editor,
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height,
            char_width,
            background_color: Color::from_rgb(0.15, 0.15, 0.15),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            cursor_color: Color::from_rgb(1.0, 1.0, 1.0),
            selection_color: Color::from_rgba(0.3, 0.5, 1.0, 0.3),
            shortcut_manager: ShortcutManager::new(),
            on_message: Box::new(on_message),
        };

        // Note: Parent application should call editor.set_char_dimensions()
        // with widget.char_dimensions() to update the core editor
        widget
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        let (char_width, line_height) = Self::measure_char_dimensions(size);
        self.line_height = line_height;
        self.char_width = char_width;

        // Note: Parent application should call editor.set_char_dimensions()
        // with self.char_dimensions() to update the core editor
        self
    }

    pub fn colors(
        mut self,
        background: Color,
        text: Color,
        cursor: Color,
        selection: Color,
    ) -> Self {
        self.background_color = background;
        self.text_color = text;
        self.cursor_color = cursor;
        self.selection_color = selection;
        self
    }

    /// Measure character dimensions for the given font size
    /// Uses improved calculations based on typical monospace font characteristics
    fn measure_char_dimensions(font_size: f32) -> (f32, f32) {
        // For monospace fonts, character width is typically around 0.6 times font size
        // This is more accurate than the previous hardcoded 8.0 value
        let char_width = font_size * 0.6;

        // Line height should be slightly larger than font size for readability
        // 1.2-1.4 is typical, we use 1.3 as a good middle ground
        let line_height = font_size * 1.3;

        // Ensure minimum values to prevent layout issues
        let char_width = char_width.max(1.0);
        let line_height = line_height.max(font_size);

        (char_width, line_height)
    }

    /// Get the current character dimensions for use by the parent application
    /// The parent should call editor.set_char_dimensions() with these values
    pub fn char_dimensions(&self) -> (f32, f32) {
        (self.char_width, self.line_height)
    }

    /// Convert screen point to editor position (line/column)
    fn point_to_position(&self, point: Point) -> Position {
        let viewport = self.editor.viewport();

        // Find the line that contains this Y position using actual line positions
        let target_y = point.y; // point.y is already relative to widget bounds
        let visible_lines_with_partial = self.editor.get_visible_lines_with_partial();

        let line = if !visible_lines_with_partial.is_empty() {
            let mut found_line = 0;
            let mut best_distance = f32::INFINITY;

            // Find the closest line by Y position
            for (_, partial_line) in &visible_lines_with_partial {
                let line_y = partial_line.y_offset;
                let distance = (target_y - line_y).abs();

                if distance < best_distance {
                    best_distance = distance;
                    found_line = partial_line.line_index;
                }

                // If we're within the line's bounds, use it
                if target_y >= line_y && target_y < line_y + self.line_height {
                    found_line = partial_line.line_index;
                    break;
                }
            }

            found_line
        } else {
            // Fallback to simple calculation if no visible lines available
            ((point.y + viewport.scroll_offset.1) / self.line_height).max(0.0) as usize
        };

        // Clamp to valid buffer bounds
        let buffer = self.editor.current_buffer();
        let max_line = buffer.line_count().saturating_sub(1);
        let clamped_line = line.min(max_line);

        // Get actual line content and calculate proper column position
        let clamped_column = if let Some(line_rope) = buffer.rope().get_line(clamped_line) {
            let line_str = line_rope.to_string();
            let line_content = line_str.trim_end_matches('\n');
            let line_length = line_content.chars().count();

            if line_length == 0 {
                0
            } else {
                let click_x = point.x + viewport.scroll_offset.0;
                let mut pixel_pos = 0.0;
                let mut column = 0;
                let mut found = false;

                for ch in line_content.chars() {
                    let char_width = if ch == '\t' {
                        self.char_width * 4.0
                    } else {
                        self.char_width
                    };
                    // If click is before the center of this char, stop
                    if click_x < pixel_pos + char_width / 2.0 {
                        found = true;
                        break;
                    }
                    pixel_pos += char_width;
                    column += 1;
                }
                // If click is past the last character, allow placing at end
                if !found {
                    line_length
                } else {
                    column
                }
            }
        } else {
            0
        };

        Position::new(clamped_line, clamped_column)
    }

    /// Convert Iced key events to core KeyEvent
    fn convert_key_event(
        &self,
        key: &iced::keyboard::Key<String>,
        modifiers: &iced::keyboard::Modifiers,
        text: Option<&str>,
    ) -> Option<KeyEvent> {
        // Convert modifiers
        let core_modifiers = Modifiers {
            shift: modifiers.shift(),
            control: modifiers.control(),
            alt: modifiers.alt(),
            super_key: modifiers.logo(),
        };

        // Convert key
        let core_key = match key {
            iced::keyboard::Key::Named(named) => match named {
                iced::keyboard::key::Named::Backspace => Some(Key::Named(NamedKey::Backspace)),
                iced::keyboard::key::Named::Delete => Some(Key::Named(NamedKey::Delete)),
                iced::keyboard::key::Named::ArrowLeft => Some(Key::Named(NamedKey::ArrowLeft)),
                iced::keyboard::key::Named::ArrowRight => Some(Key::Named(NamedKey::ArrowRight)),
                iced::keyboard::key::Named::ArrowUp => Some(Key::Named(NamedKey::ArrowUp)),
                iced::keyboard::key::Named::ArrowDown => Some(Key::Named(NamedKey::ArrowDown)),
                iced::keyboard::key::Named::Home => Some(Key::Named(NamedKey::Home)),
                iced::keyboard::key::Named::End => Some(Key::Named(NamedKey::End)),
                iced::keyboard::key::Named::Enter => Some(Key::Named(NamedKey::Enter)),
                iced::keyboard::key::Named::Escape => Some(Key::Named(NamedKey::Escape)),
                iced::keyboard::key::Named::Tab => Some(Key::Named(NamedKey::Tab)),
                iced::keyboard::key::Named::Space => Some(Key::Named(NamedKey::Space)),
                iced::keyboard::key::Named::PageUp => Some(Key::Named(NamedKey::PageUp)),
                iced::keyboard::key::Named::PageDown => Some(Key::Named(NamedKey::PageDown)),
                iced::keyboard::key::Named::Insert => Some(Key::Named(NamedKey::Insert)),
                iced::keyboard::key::Named::F1 => Some(Key::Named(NamedKey::F1)),
                iced::keyboard::key::Named::F2 => Some(Key::Named(NamedKey::F2)),
                iced::keyboard::key::Named::F3 => Some(Key::Named(NamedKey::F3)),
                iced::keyboard::key::Named::F4 => Some(Key::Named(NamedKey::F4)),
                iced::keyboard::key::Named::F5 => Some(Key::Named(NamedKey::F5)),
                iced::keyboard::key::Named::F6 => Some(Key::Named(NamedKey::F6)),
                iced::keyboard::key::Named::F7 => Some(Key::Named(NamedKey::F7)),
                iced::keyboard::key::Named::F8 => Some(Key::Named(NamedKey::F8)),
                iced::keyboard::key::Named::F9 => Some(Key::Named(NamedKey::F9)),
                iced::keyboard::key::Named::F10 => Some(Key::Named(NamedKey::F10)),
                iced::keyboard::key::Named::F11 => Some(Key::Named(NamedKey::F11)),
                iced::keyboard::key::Named::F12 => Some(Key::Named(NamedKey::F12)),
                _ => None,
            },
            iced::keyboard::Key::Character(c) => {
                if let Some(text) = text {
                    if text.len() == 1 {
                        Some(Key::Character(text.chars().next().unwrap()))
                    } else {
                        None
                    }
                } else {
                    if c.len() == 1 {
                        Some(Key::Character(c.chars().next().unwrap()))
                    } else {
                        None
                    }
                }
            }
            _ => None,
        };

        core_key.map(|key| KeyEvent::new(key, core_modifiers))
    }
}

/// Widget state for tracking viewport initialization
#[derive(Debug, Clone, Default)]
struct WidgetState {
    viewport: Rectangle,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for EditorWidget<'_, Message>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<WidgetState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(WidgetState::default())
    }

    fn size(&self) -> Size<Length> {
        // Fill width, but use content height for height
        let line_count = self.editor.current_buffer().line_count();
        let content_height = (line_count as f32 * self.line_height).max(self.line_height);
        Size::new(Length::Fill, Length::Fixed(content_height))
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Use the minimum of content height and the parent's max height
        let line_count = self.editor.current_buffer().line_count();
        let content_height = (line_count as f32 * self.line_height).max(self.line_height);
        let max = limits.max();
        let width = max.width;
        let height = content_height.min(max.height);
        let size = Size::new(width, height);
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Create optimized renderer for this frame
        let mut optimized_renderer = EditorRenderer::new(
            self.font_size,
            self.line_height,
            self.char_width,
            self.background_color,
            self.text_color,
            self.cursor_color,
            self.selection_color,
        );

        // Use the extremely optimized renderer
        optimized_renderer.render(self.editor, renderer, bounds);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        // Check if viewport needs to be initialized
        let widget_state = tree.state.downcast_mut::<WidgetState>();
        let bounds = layout.bounds();
        if widget_state.viewport != bounds {
            widget_state.viewport = bounds;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    // Convert cursor position to relative coordinates within the widget
                    let bounds = layout.bounds();
                    let relative_position =
                        Point::new(cursor_position.x - bounds.x, cursor_position.y - bounds.y);
                    let editor_position = self.point_to_position(relative_position);
                    let message = (self.on_message)(EditorMessage::MoveCursorTo(editor_position));
                    shell.publish(message);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    // Convert cursor position to relative coordinates within the widget
                    let bounds = layout.bounds();
                    let relative_position =
                        Point::new(cursor_position.x - bounds.x, cursor_position.y - bounds.y);
                    let editor_position = self.point_to_position(relative_position);
                    let message = (self.on_message)(EditorMessage::MoveCursorTo(editor_position));
                    shell.publish(message);
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // TODO: Handle selection
            }
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => {
                // Convert key events to core KeyEvent and handle with shortcut manager
                let key_string = match key {
                    iced::keyboard::Key::Named(named) => iced::keyboard::Key::Named(*named),
                    iced::keyboard::Key::Character(smol_str) => {
                        iced::keyboard::Key::Character(smol_str.to_string())
                    }
                    iced::keyboard::Key::Unidentified => iced::keyboard::Key::Unidentified,
                };
                if let Some(key_event) =
                    self.convert_key_event(&key_string, &modifiers, text.as_deref())
                {
                    if let Some(shortcut_event) = self.shortcut_manager.handle_key_event(key_event)
                    {
                        let message = (self.on_message)(shortcut_event);
                        shell.publish(message);
                    }
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_delta = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        // Reduce the multiplier for smoother scrolling with partial lines
                        Vector::new(0.0, -y * self.line_height * 0.8)
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => Vector::new(0.0, -y * 0.5),
                };

                // Calculate scroll bounds using buffer information
                let bounds = layout.bounds();

                let message = EditorMessage::Scroll(scroll_delta.x, scroll_delta.y);
                shell.publish((self.on_message)(message));
                let message = EditorMessage::UpdateViewport(bounds.width, bounds.height);
                shell.publish((self.on_message)(message));
            }
            _ => {}
        }
    }
}

/// Helper function to create the editor widget as an Element
pub fn editor_widget<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    on_message: impl Fn(EditorMessage) -> Message + 'static,
) -> Element<'a, Message, Theme, iced::Renderer> {
    Element::new(EditorWidget::new(editor, on_message))
}

/// Get measured character dimensions for a given font size
///
/// Returns (char_width, line_height) tuple with dimensions calculated based on
/// the actual font characteristics rather than hardcoded values.
///
/// This function should be used by parent applications to update the core editor's
/// viewport by calling `editor.set_char_dimensions(char_width, line_height)`.
///
/// # Example
/// ```rust
/// let font_size = 16.0;
/// let (char_width, line_height) = get_char_dimensions(font_size);
/// editor.set_char_dimensions(char_width, line_height);
/// ```
pub fn get_char_dimensions(font_size: f32) -> (f32, f32) {
    EditorWidget::<()>::measure_char_dimensions(font_size)
}

/// Convenience function to create a styled editor widget
///
/// # Important
/// When using this widget, you should update the core editor's character dimensions
/// by calling `editor.set_char_dimensions()` with the values from `get_char_dimensions()`.
/// This ensures the editor's viewport calculations match the widget's text rendering.
///
/// # Example
/// ```rust
/// let font_size = 16.0;
/// let (char_width, line_height) = get_char_dimensions(font_size);
/// editor.set_char_dimensions(char_width, line_height);
/// let widget = styled_editor(&editor, font_size, true, Message::Widget);
/// ```
pub fn styled_editor<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    font_size: f32,
    dark_theme: bool,
    on_message: impl Fn(EditorMessage) -> Message + 'static,
) -> Element<'a, Message, Theme, iced::Renderer> {
    let widget = if dark_theme {
        EditorWidget::new(editor, on_message)
            .font_size(font_size)
            .colors(
                Color::from_rgb(0.12, 0.12, 0.12),    // Dark background
                Color::from_rgb(0.9, 0.9, 0.9),       // Light text
                Color::from_rgb(1.0, 1.0, 1.0),       // White cursor
                Color::from_rgba(0.3, 0.5, 1.0, 0.3), // Blue selection
            )
    } else {
        EditorWidget::new(editor, on_message)
            .font_size(font_size)
            .colors(
                Color::WHITE,                         // Light background
                Color::BLACK,                         // Dark text
                Color::BLACK,                         // Black cursor
                Color::from_rgba(0.3, 0.5, 1.0, 0.3), // Blue selection
            )
    };

    Element::new(widget)
}
