use iced::{
    advanced::{
        layout::{self, Layout},
        renderer::{self, Quad},
        widget::{self, Widget},
        Clipboard, Shell,
    },
    mouse, Color, Element, Event, Font, Length, Point, Rectangle, Size, Theme, Vector,
};
use icedit_core::{
    Editor, Key, KeyEvent, Modifiers, NamedKey, Position, Selection, ShortcutEvent, ShortcutManager,
};

// Use Iced's SmolStr directly
type IcedSmolStr = iced::advanced::graphics::core::SmolStr;

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
    /// Shortcut event from the editor
    ShortcutEvent(ShortcutEvent),
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
    on_message: Box<dyn Fn(WidgetMessage) -> Message>,
}

impl<'a, Message> EditorWidget<'a, Message> {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const DEFAULT_LINE_HEIGHT: f32 = 18.0;
    const DEFAULT_CHAR_WIDTH: f32 = 8.0;

    pub fn new<F>(editor: &'a Editor, on_message: F) -> Self
    where
        F: Fn(WidgetMessage) -> Message + 'static,
    {
        Self {
            editor,
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::DEFAULT_LINE_HEIGHT,
            char_width: Self::DEFAULT_CHAR_WIDTH,
            background_color: Color::from_rgb(0.15, 0.15, 0.15),
            text_color: Color::from_rgb(0.9, 0.9, 0.9),
            cursor_color: Color::from_rgb(1.0, 1.0, 1.0),
            selection_color: Color::from_rgba(0.3, 0.5, 1.0, 0.3),
            shortcut_manager: ShortcutManager::new(),
            on_message: Box::new(on_message),
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self.line_height = size * 1.3;
        self.char_width = size * 0.6;
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

    /// Get the actual Y position of a line using the same logic as the text renderer
    fn get_line_y_position(&self, line_index: usize) -> f32 {
        let visible_lines_with_partial = self.editor.get_visible_lines_with_partial();

        // Find the line in the visible lines
        for (_, partial_line) in &visible_lines_with_partial {
            if partial_line.line_index == line_index {
                return partial_line.y_offset;
            }
        }

        // If not found in visible lines, calculate based on line height
        // This is a fallback for lines that might not be visible
        line_index as f32 * self.line_height
    }

    fn position_to_point(&self, position: Position) -> Point {
        let viewport = self.editor.viewport();

        // Calculate x position by measuring actual character widths
        let x_pos =
            if let Some(line_rope) = self.editor.current_buffer().rope().get_line(position.line) {
                let line_str = line_rope.to_string();
                let line_content = line_str.trim_end_matches('\n');

                let mut pixel_pos = 0.0;
                let mut char_count = 0;

                for ch in line_content.chars() {
                    if char_count >= position.column {
                        break;
                    }

                    let char_width = if ch == '\t' {
                        // Tab width is typically 4 or 8 characters
                        self.char_width * 4.0
                    } else {
                        self.char_width
                    };

                    pixel_pos += char_width;
                    char_count += 1;
                }

                pixel_pos
            } else {
                // Invalid line, position at start
                0.0
            };

        // Use the same line positioning logic as the text renderer
        let y_pos = self.get_line_y_position(position.line);

        Point::new(
            x_pos - viewport.scroll_offset.0,
            y_pos, // y_offset is already relative to widget bounds
        )
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

        // Calculate initial column based on x position
        let raw_column = ((point.x + viewport.scroll_offset.0) / self.char_width).max(0.0) as usize;

        // Clamp to valid buffer bounds
        let buffer = self.editor.current_buffer();
        let max_line = buffer.line_count().saturating_sub(1);
        let clamped_line = line.min(max_line);

        // Get actual line content and calculate proper column position
        let clamped_column = if let Some(line_rope) = buffer.rope().get_line(clamped_line) {
            let line_str = line_rope.to_string();
            // Remove trailing newline if present for accurate length calculation
            let line_content = line_str.trim_end_matches('\n');
            let line_length = line_content.chars().count();

            if line_length == 0 {
                // Empty line, position at start
                0
            } else {
                // For non-empty lines, find the closest valid character position
                let mut char_pos = 0;
                let mut pixel_pos = 0.0;

                for (i, ch) in line_content.char_indices() {
                    let char_width = if ch == '\t' {
                        // Tab width is typically 4 or 8 characters
                        self.char_width * 4.0
                    } else {
                        self.char_width
                    };

                    // Check if we're past the click position
                    if pixel_pos + char_width / 2.0 > point.x + viewport.scroll_offset.0 {
                        break;
                    }

                    char_pos = i + ch.len_utf8();
                    pixel_pos += char_width;
                }

                // Convert byte position to character position
                let char_column = line_content[..char_pos].chars().count();
                char_column.min(line_length)
            }
        } else {
            // Invalid line, position at start
            0
        };

        Position::new(clamped_line, clamped_column)
    }

    /// Convert Iced key events to core KeyEvent
    fn convert_key_event(
        &self,
        key: &iced::keyboard::Key<IcedSmolStr>,
        modifiers: iced::keyboard::Modifiers,
        text: Option<IcedSmolStr>,
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
                if let Some(text) = &text {
                    let text_str = text.as_str();
                    if text_str.len() == 1 {
                        Some(Key::Character(text_str.chars().next().unwrap()))
                    } else {
                        None
                    }
                } else {
                    let c_str = c.as_str();
                    if c_str.len() == 1 {
                        Some(Key::Character(c_str.chars().next().unwrap()))
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for EditorWidget<'_, Message>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Draw background
        renderer.fill_quad(
            Quad {
                bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            },
            self.background_color,
        );

        // Get visible lines with partial line information for smooth scrolling
        let visible_lines_with_partial = self.editor.get_visible_lines_with_partial();

        // Draw selection with support for partial lines
        if let Some(selection) = self.editor.current_selection() {
            let start_point = self.position_to_point(selection.start);
            let end_point = self.position_to_point(selection.end);

            if selection.start.line == selection.end.line {
                // Single line selection
                let selection_bounds = Rectangle::new(
                    Point::new(start_point.x + bounds.x, start_point.y + bounds.y),
                    Size::new(end_point.x - start_point.x, self.line_height),
                );
                renderer.fill_quad(
                    Quad {
                        bounds: selection_bounds,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    },
                    self.selection_color,
                );
            } else {
                // Multi-line selection - use partial line information for proper clipping
                for (_line_content, partial_line) in &visible_lines_with_partial {
                    let line_idx = partial_line.line_index;
                    if line_idx >= selection.start.line && line_idx <= selection.end.line {
                        let (start_x, end_x) = if line_idx == selection.start.line {
                            (
                                selection.start.column as f32 * self.char_width,
                                bounds.width,
                            )
                        } else if line_idx == selection.end.line {
                            (0.0, selection.end.column as f32 * self.char_width)
                        } else {
                            (0.0, bounds.width)
                        };

                        // Calculate the visible portion of the selection for this line
                        let line_y = bounds.y + partial_line.y_offset;
                        let visible_height =
                            self.line_height - partial_line.clip_top - partial_line.clip_bottom;
                        let selection_y = line_y + partial_line.clip_top;

                        let selection_bounds = Rectangle::new(
                            Point::new(
                                start_x + bounds.x - self.editor.viewport().scroll_offset.0,
                                selection_y,
                            ),
                            Size::new(end_x - start_x, visible_height),
                        );
                        renderer.fill_quad(
                            Quad {
                                bounds: selection_bounds,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            },
                            self.selection_color,
                        );
                    }
                }
            }
        }

        // Draw text using visible lines with partial line support for smooth scrolling
        for (line_content, partial_line) in visible_lines_with_partial.iter() {
            let position = Point::new(
                bounds.x - self.editor.viewport().scroll_offset.0,
                bounds.y + partial_line.y_offset,
            );

            // Calculate visible height for this line
            let visible_height =
                self.line_height - partial_line.clip_top - partial_line.clip_bottom;

            // Create a clipped rendering area for the text
            let text_bounds = Rectangle::new(
                Point::new(position.x, position.y + partial_line.clip_top),
                Size::new(bounds.width, visible_height),
            );

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: line_content.to_string(),
                    bounds: Size::new(bounds.width, self.line_height),
                    size: iced::Pixels(self.font_size),
                    font: Font::MONOSPACE,
                    horizontal_alignment: iced::alignment::Horizontal::Left,
                    vertical_alignment: iced::alignment::Vertical::Top,
                    line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(
                        self.line_height,
                    )),
                    shaping: iced::advanced::text::Shaping::Advanced,
                    wrapping: iced::advanced::text::Wrapping::None,
                },
                position,
                self.text_color,
                text_bounds, // Use clipped bounds instead of full bounds
            );
        }

        // Draw cursor with proper height based on font size
        let cursor_position = self.editor.current_cursor().position();
        let cursor_point = self.position_to_point(cursor_position);

        // Calculate cursor dimensions based on font size
        let cursor_width = 2.0;
        let cursor_height = self.font_size * 1.2; // Slightly taller than font for better visibility

        // Only draw cursor if it's visible in the viewport
        let cursor_screen_x = cursor_point.x + bounds.x;
        let cursor_screen_y = cursor_point.y + bounds.y;

        // Check if cursor is within visible bounds (with some tolerance for cursor width)
        if cursor_screen_x >= bounds.x - cursor_width
            && cursor_screen_x <= bounds.x + bounds.width
            && cursor_screen_y >= bounds.y
            && cursor_screen_y + cursor_height <= bounds.y + bounds.height
        {
            let cursor_bounds = Rectangle::new(
                Point::new(cursor_screen_x, cursor_screen_y),
                Size::new(cursor_width, cursor_height),
            );
            renderer.fill_quad(
                Quad {
                    bounds: cursor_bounds,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                },
                self.cursor_color,
            );
        }
    }

    fn on_event(
        &mut self,
        _tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        use iced::advanced::graphics::core::event::Status;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    // Convert cursor position to relative coordinates within the widget
                    let bounds = layout.bounds();
                    let relative_position =
                        Point::new(cursor_position.x - bounds.x, cursor_position.y - bounds.y);
                    let editor_position = self.point_to_position(relative_position);
                    let message = (self.on_message)(WidgetMessage::MousePressed(editor_position));
                    shell.publish(message);
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    // Convert cursor position to relative coordinates within the widget
                    let bounds = layout.bounds();
                    let relative_position =
                        Point::new(cursor_position.x - bounds.x, cursor_position.y - bounds.y);
                    let editor_position = self.point_to_position(relative_position);
                    let message = (self.on_message)(WidgetMessage::MouseReleased(editor_position));
                    shell.publish(message);
                    return Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                    // Convert cursor position to relative coordinates within the widget
                    let bounds = layout.bounds();
                    let relative_position =
                        Point::new(cursor_position.x - bounds.x, cursor_position.y - bounds.y);
                    let editor_position = self.point_to_position(relative_position);
                    let message = (self.on_message)(WidgetMessage::MouseMoved(editor_position));
                    shell.publish(message);
                    return Status::Captured;
                }
            }
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => {
                // Convert key events to core KeyEvent and handle with shortcut manager
                if let Some(key_event) = self.convert_key_event(&key, modifiers, text) {
                    if let Some(shortcut_event) = self.shortcut_manager.handle_key_event(key_event)
                    {
                        let message =
                            (self.on_message)(WidgetMessage::ShortcutEvent(shortcut_event));
                        shell.publish(message);
                        return Status::Captured;
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
                let line_count = self.editor.current_buffer().line_count();
                let content_height = line_count as f32 * self.line_height;

                // Estimate content width based on viewport or use a reasonable default
                let estimated_max_line_width = 120; // characters
                let content_width = estimated_max_line_width as f32 * self.char_width;

                let scroll_bounds = ScrollBounds {
                    content_size: Size::new(content_width, content_height),
                    viewport_size: Size::new(bounds.width, bounds.height),
                    line_height: self.line_height,
                };

                let message = (self.on_message)(WidgetMessage::Scroll(scroll_delta, scroll_bounds));
                shell.publish(message);
                return Status::Captured;
            }
            _ => {}
        }

        Status::Ignored
    }
}

/// Helper function to create the editor widget as an Element
pub fn editor_widget<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    on_message: impl Fn(WidgetMessage) -> Message + 'static,
) -> Element<'a, Message, Theme, iced::Renderer> {
    Element::new(EditorWidget::new(editor, on_message))
}

/// Convenience function to create a styled editor widget
pub fn styled_editor<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    font_size: f32,
    dark_theme: bool,
    on_message: impl Fn(WidgetMessage) -> Message + 'static,
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
