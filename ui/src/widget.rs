use crate::{renderer::EditorRenderer, utils, Viewport};
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
        Self {
            cursor_position: editor.current_cursor().position(),
            selection: editor.current_selection().cloned(),
            scroll_offset: Vector::ZERO,
        }
    }
}

/// Messages that the widget can emit - these will be routed by the user
#[derive(Debug, Clone)]
pub enum WidgetMessage {
    EditorMessage(EditorMessage),
    MousePressed(Position),
    MouseReleased(Position),
    MouseMoved(Position),
}

/// Widget state for tracking viewport initialization and mouse drag state
#[derive(Debug, Default)]
struct WidgetState {
    /// Tracks the last known viewport bounds for initialization detection
    viewport_bounds: Rectangle,
    /// The editor viewport that manages scrolling and visible content
    viewport: Viewport,
    /// Tracks if we're currently dragging (selecting with mouse)
    is_dragging: bool,
    /// The position where dragging started (for selection anchor)
    drag_start_position: Option<Position>,
    /// Current mouse position for auto-scroll calculations
    current_mouse_position: Option<Point>,
    /// Current auto-scroll delta for smooth scrolling
    auto_scroll_delta: Vector,
    /// Whether auto-scrolling is currently active
    is_auto_scrolling: bool,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for EditorWidget<'_, Message>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    Message: Clone,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<WidgetState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(WidgetState::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.max();
        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let widget_state = tree.state.downcast_ref::<WidgetState>();

        // Create renderer with current styling
        let mut editor_renderer = EditorRenderer::new(
            self.font_size,
            self.line_height,
            self.char_width,
            self.background_color,
            self.text_color,
            self.cursor_color,
            self.selection_color,
        );

        // Render the editor content
        editor_renderer.render(self.editor, &widget_state.viewport, renderer, bounds);
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
        let widget_state = tree.state.downcast_mut::<WidgetState>();
        let bounds = layout.bounds();

        // Check if viewport needs to be initialized or updated
        if widget_state.viewport_bounds.size() != bounds.size() {
            widget_state.viewport_bounds = bounds;
            widget_state
                .viewport
                .set_char_dimensions(self.char_width, self.line_height);
            widget_state.viewport.set_size(bounds.width, bounds.height);
        }

        // Handle continuous auto-scrolling if we're dragging outside bounds
        if widget_state.is_auto_scrolling && widget_state.is_dragging {
            self.apply_auto_scroll_and_selection(widget_state, shell, bounds);
            // Request another update to continue auto-scrolling
            shell.request_redraw();
        }

        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(position) = cursor.position_in(bounds) {
                            let editor_position =
                                self.point_to_position(position, &widget_state.viewport);

                            // Start dragging for selection
                            widget_state.is_dragging = true;
                            widget_state.drag_start_position = Some(editor_position);
                            widget_state.current_mouse_position = cursor.position();

                            // Move cursor to clicked position
                            let message =
                                (self.on_message)(EditorMessage::MoveCursorTo(editor_position));
                            shell.publish(message);
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        widget_state.is_dragging = false;
                        widget_state.drag_start_position = None;
                        widget_state.current_mouse_position = None;
                        widget_state.is_auto_scrolling = false;
                    }
                    mouse::Event::CursorMoved { .. } => {
                        widget_state.current_mouse_position = cursor.position();

                        if widget_state.is_dragging {
                            if let Some(position) = cursor.position_in(bounds) {
                                // Mouse is within bounds - stop auto-scrolling
                                widget_state.is_auto_scrolling = false;
                                widget_state.auto_scroll_delta = Vector::ZERO;

                                let editor_position =
                                    self.point_to_position(position, &widget_state.viewport);

                                // Update selection
                                if let Some(start_pos) = widget_state.drag_start_position {
                                    let message = (self.on_message)(EditorMessage::SetSelection(
                                        start_pos,
                                        editor_position,
                                    ));
                                    shell.publish(message);
                                }
                            } else {
                                // Mouse is outside bounds - calculate and apply auto-scroll
                                self.calculate_auto_scroll_delta(widget_state, bounds);
                                self.apply_auto_scroll_and_selection(widget_state, shell, bounds);
                            }
                        }
                    }
                    mouse::Event::WheelScrolled { delta } => {
                        if cursor.is_over(bounds) {
                            let scroll_delta = match delta {
                                mouse::ScrollDelta::Lines { x, y } => Vector::new(
                                    *x * self.char_width * 3.0,
                                    *y * self.line_height * 3.0,
                                ),
                                mouse::ScrollDelta::Pixels { x, y } => Vector::new(*x, *y),
                            };

                            // Apply scroll to viewport
                            let new_offset = (
                                widget_state.viewport.scroll_offset.0 - scroll_delta.x,
                                widget_state.viewport.scroll_offset.1 - scroll_delta.y,
                            );

                            // Clamp scroll offset to reasonable bounds
                            let buffer = self.editor.current_buffer();
                            let line_count = buffer.line_count();
                            let content_height = line_count as f32 * self.line_height;
                            let max_scroll_y = (content_height - bounds.height).max(0.0);
                            let max_content_width = self.calculate_max_content_width();
                            let max_scroll_x = (max_content_width - bounds.width).max(0.0);

                            let clamped_offset = (
                                new_offset.0.max(0.0).min(max_scroll_x),
                                new_offset.1.max(0.0).min(max_scroll_y),
                            );

                            widget_state
                                .viewport
                                .set_scroll_offset(clamped_offset.0, clamped_offset.1);

                            // Request redraw to show the new scroll position
                            shell.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            Event::Keyboard(keyboard_event) => match keyboard_event {
                iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                    if let Some(editor_message) = self.handle_keyboard_input(key, modifiers) {
                        let message = (self.on_message)(editor_message);
                        shell.publish(message);
                    }
                }
                _ => {}
            },
            _ => {}
        }
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

        widget
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        let (char_width, line_height) = Self::measure_char_dimensions(size);
        self.line_height = line_height;
        self.char_width = char_width;
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
        utils::calculate_char_dimensions(font_size)
    }

    /// Get the current character dimensions for use by the parent application
    pub fn char_dimensions(&self) -> (f32, f32) {
        (self.char_width, self.line_height)
    }

    /// Calculate the maximum content width for horizontal scroll limiting
    fn calculate_max_content_width(&self) -> f32 {
        utils::calculate_max_content_width(self.editor, self.char_width, 1000)
    }

    /// Convert screen point to editor position (line/column)
    fn point_to_position(&self, point: Point, viewport: &Viewport) -> Position {
        // Find the line that contains this Y position
        let target_y = point.y; // point.y is already relative to widget bounds

        let line = if !viewport.partial_lines.is_empty() {
            let mut found_line = 0;

            // Find the line that contains this Y position (not just closest)
            for partial_line in &viewport.partial_lines {
                let line_y = partial_line.y_offset;
                let line_bottom = line_y + self.line_height;

                // Check if the click is within this line's bounds
                if target_y >= line_y && target_y < line_bottom {
                    found_line = partial_line.line_index;
                    break;
                }

                // If we're past this line, update found_line in case this is the last one
                if target_y >= line_bottom {
                    found_line = partial_line.line_index;
                }
            }

            found_line
        } else {
            // Fallback to simple calculation
            ((point.y + viewport.scroll_offset.1) / self.line_height).max(0.0) as usize
        };

        // Calculate column based on X position
        let rope = self.editor.current_buffer().rope();
        let column = if line < rope.len_lines() {
            if let Some(line_text) = rope.get_line(line) {
                let line_str = line_text.to_string();
                let click_x = point.x + viewport.scroll_offset.0;

                // Find the character position that's closest to the click
                let mut current_x = 0.0;
                let mut column = 0;
                let tab_width = utils::get_tab_width(self.char_width);

                for ch in line_str.chars() {
                    let char_width = if ch == '\t' {
                        // Calculate tab width
                        let tab_stop = ((current_x / tab_width).floor() + 1.0) * tab_width;
                        tab_stop - current_x
                    } else {
                        self.char_width
                    };
                    if ch == '\n' {
                        break;
                    }

                    // Check if click is before the middle of this character
                    if click_x < current_x + char_width / 2.0 {
                        break;
                    }

                    current_x += char_width;
                    column += 1;
                }

                column
            } else {
                0
            }
        } else {
            0
        };

        Position::new(line, column)
    }

    /// Handle keyboard input and convert to editor messages
    fn handle_keyboard_input<T: AsRef<str>>(
        &self,
        key: &iced::keyboard::Key<T>,
        modifiers: &iced::keyboard::Modifiers,
    ) -> Option<EditorMessage> {
        // Convert iced key event to our key event format
        if let Some(key_event) = self.convert_key_event(key, modifiers) {
            // Handle with shortcut manager - it already handles character input properly
            self.shortcut_manager.handle_key_event(key_event)
        } else {
            None
        }
    }

    fn convert_key_event<T: AsRef<str>>(
        &self,
        key: &iced::keyboard::Key<T>,
        modifiers: &iced::keyboard::Modifiers,
    ) -> Option<KeyEvent> {
        let key_code = match key {
            iced::keyboard::Key::Character(c) => {
                let c_str = c.as_ref();
                if let Some(ch) = c_str.chars().next() {
                    Key::Character(ch)
                } else {
                    return None;
                }
            }
            iced::keyboard::Key::Named(named) => match named {
                iced::keyboard::key::Named::ArrowUp => Key::Named(NamedKey::ArrowUp),
                iced::keyboard::key::Named::ArrowDown => Key::Named(NamedKey::ArrowDown),
                iced::keyboard::key::Named::ArrowLeft => Key::Named(NamedKey::ArrowLeft),
                iced::keyboard::key::Named::ArrowRight => Key::Named(NamedKey::ArrowRight),
                iced::keyboard::key::Named::Backspace => Key::Named(NamedKey::Backspace),
                iced::keyboard::key::Named::Delete => Key::Named(NamedKey::Delete),
                iced::keyboard::key::Named::Enter => Key::Named(NamedKey::Enter),
                iced::keyboard::key::Named::Tab => Key::Named(NamedKey::Tab),
                iced::keyboard::key::Named::Space => Key::Named(NamedKey::Space),
                iced::keyboard::key::Named::Home => Key::Named(NamedKey::Home),
                iced::keyboard::key::Named::End => Key::Named(NamedKey::End),
                iced::keyboard::key::Named::PageUp => Key::Named(NamedKey::PageUp),
                iced::keyboard::key::Named::PageDown => Key::Named(NamedKey::PageDown),
                iced::keyboard::key::Named::Escape => Key::Named(NamedKey::Escape),
                _ => return None,
            },
            _ => return None,
        };

        let modifiers = Modifiers {
            shift: modifiers.shift(),
            control: modifiers.control(),
            alt: modifiers.alt(),
            super_key: modifiers.logo(),
        };

        Some(KeyEvent::new(key_code, modifiers))
    }

    /// Calculate auto-scroll delta based on mouse position
    fn calculate_auto_scroll_delta(&self, widget_state: &mut WidgetState, bounds: Rectangle) {
        if let Some(mouse_pos) = widget_state.current_mouse_position {
            let mut delta = Vector::ZERO;
            const SCROLL_MARGIN: f32 = 50.0;
            const BASE_SCROLL_SPEED: f32 = 3.0;

            // Calculate distance-based scroll speed for vertical scrolling
            if mouse_pos.y < bounds.y {
                let distance = bounds.y - mouse_pos.y;
                let speed = BASE_SCROLL_SPEED * (1.0 + distance / SCROLL_MARGIN).min(5.0);
                delta.y = -speed;
            } else if mouse_pos.y > bounds.y + bounds.height {
                let distance = mouse_pos.y - (bounds.y + bounds.height);
                let speed = BASE_SCROLL_SPEED * (1.0 + distance / SCROLL_MARGIN).min(5.0);
                delta.y = speed;
            }

            // Calculate distance-based scroll speed for horizontal scrolling
            if mouse_pos.x < bounds.x {
                let distance = bounds.x - mouse_pos.x;
                let speed = BASE_SCROLL_SPEED * (1.0 + distance / SCROLL_MARGIN).min(5.0);
                delta.x = -speed;
            } else if mouse_pos.x > bounds.x + bounds.width {
                let distance = mouse_pos.x - (bounds.x + bounds.width);
                let speed = BASE_SCROLL_SPEED * (1.0 + distance / SCROLL_MARGIN).min(5.0);
                delta.x = speed;
            }

            widget_state.auto_scroll_delta = delta;
            widget_state.is_auto_scrolling = delta != Vector::ZERO;
        } else {
            widget_state.auto_scroll_delta = Vector::ZERO;
            widget_state.is_auto_scrolling = false;
        }
    }

    /// Apply auto-scroll and update selection if dragging
    fn apply_auto_scroll_and_selection(
        &self,
        widget_state: &mut WidgetState,
        shell: &mut Shell<'_, Message>,
        bounds: Rectangle,
    ) {
        if widget_state.is_auto_scrolling {
            // Apply auto-scroll to viewport
            let new_offset = (
                widget_state.viewport.scroll_offset.0 + widget_state.auto_scroll_delta.x,
                widget_state.viewport.scroll_offset.1 + widget_state.auto_scroll_delta.y,
            );

            // Clamp scroll offset to reasonable bounds
            let buffer = self.editor.current_buffer();
            let line_count = buffer.line_count();
            let content_height = line_count as f32 * self.line_height;
            let max_scroll_y = (content_height - bounds.height).max(0.0);
            let max_content_width = self.calculate_max_content_width();
            let max_scroll_x = (max_content_width - bounds.width).max(0.0);

            let clamped_offset = (
                new_offset.0.max(0.0).min(max_scroll_x),
                new_offset.1.max(0.0).min(max_scroll_y),
            );

            widget_state
                .viewport
                .set_scroll_offset(clamped_offset.0, clamped_offset.1);

            // Request redraw to show the auto-scroll
            shell.request_redraw();

            // Update selection based on current mouse position
            if let Some(current_pos) = widget_state.current_mouse_position {
                // Calculate position for selection update
                let relative_pos = Point::new(current_pos.x - bounds.x, current_pos.y - bounds.y);
                let editor_position = self.point_to_position(relative_pos, &widget_state.viewport);

                // Update selection
                if let Some(start_pos) = widget_state.drag_start_position {
                    let message =
                        (self.on_message)(EditorMessage::SetSelection(start_pos, editor_position));
                    shell.publish(message);
                }
            }
        }
    }
}

impl<'a, Message: Clone + 'a> From<EditorWidget<'a, Message>>
    for Element<'a, Message, Theme, iced::Renderer>
{
    fn from(widget: EditorWidget<'a, Message>) -> Self {
        Element::new(widget)
    }
}

/// Convenience function to create an editor widget with default styling
pub fn editor_widget<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    on_message: impl Fn(EditorMessage) -> Message + 'static,
) -> Element<'a, Message, Theme, iced::Renderer> {
    Element::new(EditorWidget::new(editor, on_message))
}

/// Get character dimensions for a given font size
/// This is a utility function that can be used by applications to calculate
/// viewport by calling `editor.set_char_dimensions(char_width, line_height)`.
///
/// # Arguments
/// * `font_size` - The font size in pixels
///
/// # Returns
/// A tuple of (char_width, line_height) in pixels
///
/// # Example
/// ```rust
/// let (char_width, line_height) = get_char_dimensions(14.0);
/// editor.set_char_dimensions(char_width, line_height);
/// ```
///
/// This ensures the editor's viewport calculations match the widget's text rendering.
pub fn get_char_dimensions(font_size: f32) -> (f32, f32) {
    utils::calculate_char_dimensions(font_size)
}

/// Convenience function to create a styled editor widget
pub fn styled_editor<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    font_size: f32,
    dark_theme: bool,
    on_message: impl Fn(EditorMessage) -> Message + 'static,
) -> EditorWidget<'a, Message> {
    let colors = if dark_theme {
        (
            Color::from_rgb(0.12, 0.12, 0.15),    // background
            Color::from_rgb(0.9, 0.9, 0.9),       // text
            Color::from_rgb(1.0, 1.0, 1.0),       // cursor
            Color::from_rgba(0.3, 0.5, 1.0, 0.3), // selection
        )
    } else {
        (
            Color::from_rgb(1.0, 1.0, 1.0),       // background
            Color::from_rgb(0.1, 0.1, 0.1),       // text
            Color::from_rgb(0.0, 0.0, 0.0),       // cursor
            Color::from_rgba(0.3, 0.5, 1.0, 0.3), // selection
        )
    };

    EditorWidget::new(editor, on_message)
        .font_size(font_size)
        .colors(colors.0, colors.1, colors.2, colors.3)
}
