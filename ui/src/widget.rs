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
    pub buffer_content: String,
    pub cursor_position: Position,
    pub selection: Option<Selection>,
    pub scroll_offset: Vector,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffer_content: String::new(),
            cursor_position: Position::zero(),
            selection: None,
            scroll_offset: Vector::ZERO,
        }
    }

    pub fn from_editor(editor: &Editor) -> Self {
        Self {
            buffer_content: editor.current_buffer().text(),
            cursor_position: editor.current_cursor().position(),
            selection: editor.current_selection().cloned(),
            scroll_offset: Vector::ZERO,
        }
    }
}

/// Messages that the widget can emit - these will be routed by the user
#[derive(Debug, Clone)]
pub enum WidgetMessage {
    /// Shortcut event from the editor
    ShortcutEvent(ShortcutEvent),
    /// Scroll events
    Scroll(Vector),
    /// Mouse events
    MousePressed(Point),
    MouseReleased(Point),
    MouseMoved(Point),
}

/// The editor widget that renders text using custom drawing
pub struct EditorWidget<Message> {
    state: EditorState,
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

impl<Message> EditorWidget<Message> {
    const DEFAULT_FONT_SIZE: f32 = 14.0;
    const DEFAULT_LINE_HEIGHT: f32 = 18.0;
    const DEFAULT_CHAR_WIDTH: f32 = 8.0;

    pub fn new<F>(state: EditorState, on_message: F) -> Self
    where
        F: Fn(WidgetMessage) -> Message + 'static,
    {
        Self {
            state,
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

    fn position_to_point(&self, position: Position) -> Point {
        Point::new(
            position.column as f32 * self.char_width - self.state.scroll_offset.x,
            position.line as f32 * self.line_height - self.state.scroll_offset.y,
        )
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for EditorWidget<Message>
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

        // Calculate visible text area
        let visible_lines_start = (self.state.scroll_offset.y / self.line_height) as usize;
        let visible_lines_count = (bounds.height / self.line_height).ceil() as usize + 1;
        let visible_lines_end = visible_lines_start + visible_lines_count;

        let lines: Vec<&str> = self.state.buffer_content.lines().collect();

        // Draw selection
        if let Some(selection) = &self.state.selection {
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
                // Multi-line selection
                for line_idx in selection.start.line..=selection.end.line {
                    if line_idx >= visible_lines_start && line_idx < visible_lines_end {
                        let line_y =
                            line_idx as f32 * self.line_height - self.state.scroll_offset.y;
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

                        let selection_bounds = Rectangle::new(
                            Point::new(
                                start_x + bounds.x - self.state.scroll_offset.x,
                                line_y + bounds.y,
                            ),
                            Size::new(end_x - start_x, self.line_height),
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

        // Draw text
        for (line_idx, line) in lines.iter().enumerate() {
            if line_idx >= visible_lines_start && line_idx < visible_lines_end {
                let position = Point::new(
                    bounds.x - self.state.scroll_offset.x,
                    bounds.y + line_idx as f32 * self.line_height - self.state.scroll_offset.y,
                );

                renderer.fill_text(
                    iced::advanced::text::Text {
                        content: line.to_string(),
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
                    bounds,
                );
            }
        }

        // Draw cursor
        let cursor_point = self.position_to_point(self.state.cursor_position);
        let cursor_bounds = Rectangle::new(
            Point::new(cursor_point.x + bounds.x, cursor_point.y + bounds.y),
            Size::new(2.0, self.line_height),
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
                    let message = (self.on_message)(WidgetMessage::MousePressed(cursor_position));
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
                        Vector::new(0.0, -y * self.line_height * 3.0)
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => Vector::new(0.0, -y),
                };

                let message = (self.on_message)(WidgetMessage::Scroll(scroll_delta));
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
    state: EditorState,
    on_message: impl Fn(WidgetMessage) -> Message + 'static,
) -> Element<'a, Message, Theme, iced::Renderer> {
    Element::new(EditorWidget::new(state, on_message))
}

/// Convenience function to create a styled editor widget
pub fn styled_editor<'a, Message: 'a + Clone>(
    state: EditorState,
    font_size: f32,
    dark_theme: bool,
    on_message: impl Fn(WidgetMessage) -> Message + 'static,
) -> Element<'a, Message, Theme, iced::Renderer> {
    let widget = if dark_theme {
        EditorWidget::new(state, on_message)
            .font_size(font_size)
            .colors(
                Color::from_rgb(0.12, 0.12, 0.12),    // Dark background
                Color::from_rgb(0.9, 0.9, 0.9),       // Light text
                Color::from_rgb(1.0, 1.0, 1.0),       // White cursor
                Color::from_rgba(0.3, 0.5, 1.0, 0.3), // Blue selection
            )
    } else {
        EditorWidget::new(state, on_message)
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
