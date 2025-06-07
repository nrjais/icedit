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
            self.calculate_gutter_width(),
            self.gutter_background_color,
            self.line_number_color,
            self.current_line_number_color,
            self.gutter_padding,
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

                            // Ensure cursor is visible after mouse click
                            self.ensure_cursor_visible(widget_state, bounds, shell);
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
                        // Check if this is a cursor movement command that should ensure cursor visibility
                        let should_ensure_cursor_visible =
                            self.is_cursor_movement_command(&editor_message);

                        let message = (self.on_message)(editor_message);
                        shell.publish(message);

                        // After publishing the editor message, ensure cursor is visible if needed
                        if should_ensure_cursor_visible {
                            self.ensure_cursor_visible(widget_state, bounds, shell);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

/// The editor widget that renders text using custom drawing
///
/// This widget automatically ensures the cursor remains visible when moving
/// via keyboard navigation (arrow keys, page up/down, etc.) by scrolling
/// the viewport as needed with comfortable margins.
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
    gutter_background_color: Color,
    line_number_color: Color,
    current_line_number_color: Color,
    gutter_padding: f32,
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
            gutter_background_color: Color::from_rgba(0.2, 0.2, 0.2, 0.0),
            line_number_color: Color::from_rgb(0.7, 0.7, 0.7),
            current_line_number_color: Color::from_rgb(1.0, 0.8, 0.2),
            gutter_padding: 8.0,
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

    /// Configure the line number gutter colors
    pub fn gutter(
        mut self,
        gutter_background_color: Color,
        line_number_color: Color,
        current_line_number_color: Color,
        gutter_padding: f32,
    ) -> Self {
        self.gutter_background_color = gutter_background_color;
        self.line_number_color = line_number_color;
        self.current_line_number_color = current_line_number_color;
        self.gutter_padding = gutter_padding;
        self
    }

    /// Calculate the gutter width based on the number of lines in the editor
    pub fn calculate_gutter_width(&self) -> f32 {
        // Auto-calculate based on line count
        let line_count = self.editor.current_buffer().rope().len_lines();
        let digits = if line_count == 0 {
            1
        } else {
            (line_count as f32).log10().floor() as usize + 1
        };

        // Minimum width for 2 digits, plus padding on both sides
        let min_digits = 2.max(digits);
        (min_digits as f32 * self.char_width) + (self.gutter_padding * 2.0)
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
        let gutter_width = self.calculate_gutter_width();

        // If click is within the gutter area, position cursor at start of line
        let adjusted_point = if point.x < gutter_width {
            Point::new(0.0, point.y)
        } else {
            Point::new(point.x - gutter_width, point.y)
        };

        // Find the line that contains this Y position
        let target_y = adjusted_point.y; // adjusted_point.y is already relative to widget bounds

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

        // Calculate column based on X position with tab handling
        let rope = self.editor.current_buffer().rope();
        let column = if line < rope.len_lines() {
            if let Some(line_text) = rope.get_line(line) {
                let line_str = line_text.to_string();
                let click_x = adjusted_point.x + viewport.scroll_offset.0;

                // Use the utility function for accurate tab-aware column calculation
                utils::x_position_to_column(click_x, &line_str, self.char_width)
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

    /// Check if the editor message is a cursor movement command that should trigger cursor visibility check
    fn is_cursor_movement_command(&self, message: &EditorMessage) -> bool {
        matches!(
            message,
            EditorMessage::MoveCursor(_)
                | EditorMessage::MoveCursorWithSelection(_)
                | EditorMessage::MoveCursorTo(_)
                | EditorMessage::ScrollToLine(_)
        )
    }

    /// Check if cursor is visible with a comfortable margin
    fn is_cursor_visible_with_margin(
        &self,
        cursor_position: Position,
        viewport: &Viewport,
    ) -> bool {
        let cursor_y = cursor_position.line as f32 * self.line_height;
        let cursor_x = {
            let rope = self.editor.current_buffer().rope();
            if cursor_position.line < rope.len_lines() {
                if let Some(line) = rope.get_line(cursor_position.line) {
                    let line_str = line.to_string();
                    utils::calculate_column_x_position(
                        cursor_position.column,
                        &line_str,
                        self.char_width,
                    )
                } else {
                    cursor_position.column as f32 * self.char_width
                }
            } else {
                cursor_position.column as f32 * self.char_width
            }
        };

        // Define margins for comfortable scrolling
        let scroll_margin_v = self.line_height * 2.0;
        let scroll_margin_h = self.char_width * 4.0;

        // Check vertical visibility with margin
        let viewport_top = viewport.scroll_offset.1 + scroll_margin_v;
        let viewport_bottom = viewport.scroll_offset.1 + viewport.size.1 - scroll_margin_v;
        let cursor_line_top = cursor_y;
        let cursor_line_bottom = cursor_y + self.line_height;

        let v_visible = cursor_line_top >= viewport_top && cursor_line_bottom <= viewport_bottom;

        // Check horizontal visibility with margin
        let viewport_left = viewport.scroll_offset.0 + scroll_margin_h;
        let viewport_right = viewport.scroll_offset.0 + viewport.size.0 - scroll_margin_h;
        let cursor_right = cursor_x + self.char_width;

        let h_visible = cursor_x >= viewport_left && cursor_right <= viewport_right;

        v_visible && h_visible
    }

    /// Ensure the cursor is visible in the viewport by scrolling if necessary
    fn ensure_cursor_visible(
        &self,
        widget_state: &mut WidgetState,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        let cursor_position = self.editor.current_cursor().position();
        let viewport = &mut widget_state.viewport;

        // Quick check: if cursor is already fully visible with some margin, no need to scroll
        if self.is_cursor_visible_with_margin(cursor_position, viewport) {
            return;
        }

        // Calculate cursor position in viewport coordinates
        let cursor_y = cursor_position.line as f32 * self.line_height;
        let cursor_x = {
            let rope = self.editor.current_buffer().rope();
            if cursor_position.line < rope.len_lines() {
                if let Some(line) = rope.get_line(cursor_position.line) {
                    let line_str = line.to_string();
                    utils::calculate_column_x_position(
                        cursor_position.column,
                        &line_str,
                        self.char_width,
                    )
                } else {
                    cursor_position.column as f32 * self.char_width
                }
            } else {
                cursor_position.column as f32 * self.char_width
            }
        };

        // Calculate current scroll offset
        let mut new_scroll_x = viewport.scroll_offset.0;
        let mut new_scroll_y = viewport.scroll_offset.1;
        let mut scroll_changed = false;

        // Vertical scrolling: ensure cursor line is visible
        let viewport_top = viewport.scroll_offset.1;
        let viewport_bottom = viewport.scroll_offset.1 + viewport.size.1;
        let cursor_line_top = cursor_y;
        let cursor_line_bottom = cursor_y + self.line_height;

        // Add some margin for better UX (show a few lines above/below cursor when possible)
        let scroll_margin = self.line_height * 2.0;

        if cursor_line_top < viewport_top + scroll_margin {
            // Cursor is too close to the top or above visible area
            new_scroll_y = (cursor_line_top - scroll_margin).max(0.0);
            scroll_changed = true;
        } else if cursor_line_bottom > viewport_bottom - scroll_margin {
            // Cursor is too close to the bottom or below visible area
            new_scroll_y = cursor_line_bottom + scroll_margin - viewport.size.1;
            scroll_changed = true;
        }

        // Horizontal scrolling: ensure cursor column is visible
        let viewport_left = viewport.scroll_offset.0;
        let viewport_right = viewport.scroll_offset.0 + viewport.size.0;
        let cursor_right = cursor_x + self.char_width; // Add character width for visibility

        // Add some margin for better UX
        let h_scroll_margin = self.char_width * 4.0;

        if cursor_x < viewport_left + h_scroll_margin {
            // Cursor is too close to the left or left of visible area
            new_scroll_x = (cursor_x - h_scroll_margin).max(0.0);
            scroll_changed = true;
        } else if cursor_right > viewport_right - h_scroll_margin {
            // Cursor is too close to the right or right of visible area
            new_scroll_x = cursor_right + h_scroll_margin - viewport.size.0;
            scroll_changed = true;
        }

        // Apply scroll bounds to prevent over-scrolling
        if scroll_changed {
            let buffer = self.editor.current_buffer();
            let line_count = buffer.line_count();
            let content_height = line_count as f32 * self.line_height;
            let max_scroll_y = (content_height - bounds.height).max(0.0);
            let max_content_width = self.calculate_max_content_width();
            let max_scroll_x = (max_content_width - bounds.width).max(0.0);

            let clamped_scroll_x = new_scroll_x.max(0.0).min(max_scroll_x);
            let clamped_scroll_y = new_scroll_y.max(0.0).min(max_scroll_y);

            // Only update if the scroll position actually changed
            if (clamped_scroll_x - viewport.scroll_offset.0).abs() > 0.1
                || (clamped_scroll_y - viewport.scroll_offset.1).abs() > 0.1
            {
                viewport.set_scroll_offset(clamped_scroll_x, clamped_scroll_y);
                shell.request_redraw();
            }
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

/// Convenience function to create a styled editor widget
pub fn styled_editor<'a, Message: 'a + Clone>(
    editor: &'a Editor,
    font_size: f32,
    dark_theme: bool,
    on_message: impl Fn(EditorMessage) -> Message + 'static,
) -> EditorWidget<'a, Message> {
    let (colors, gutter_colors) = if dark_theme {
        (
            (
                Color::from_rgb(0.12, 0.12, 0.15),    // background
                Color::from_rgb(0.9, 0.9, 0.9),       // text
                Color::from_rgb(1.0, 1.0, 1.0),       // cursor
                Color::from_rgba(0.3, 0.5, 1.0, 0.3), // selection
            ),
            (
                Color::from_rgba(0.2, 0.2, 0.2, 0.0), // gutter background (more opaque)
                Color::from_rgb(0.7, 0.7, 0.7),       // line number (more visible)
                Color::from_rgb(1.0, 0.8, 0.2),       // current line number (yellow highlight)
            ),
        )
    } else {
        (
            (
                Color::from_rgb(1.0, 1.0, 1.0),       // background
                Color::WHITE,                         // text
                Color::from_rgb(0.0, 0.0, 0.0),       // cursor
                Color::from_rgba(0.3, 0.5, 1.0, 0.3), // selection
            ),
            (
                Color::from_rgba(0.9, 0.9, 0.9, 0.0), // gutter background (more opaque)
                Color::from_rgb(0.4, 0.4, 0.4),       // line number (darker, more visible)
                Color::from_rgb(0.8, 0.4, 0.0),       // current line number (orange highlight)
            ),
        )
    };

    EditorWidget::new(editor, on_message)
        .font_size(font_size)
        .colors(colors.0, colors.1, colors.2, colors.3)
        .gutter(
            gutter_colors.0, // gutter_background_color
            gutter_colors.1, // line_number_color
            gutter_colors.2, // current_line_number_color
            8.0,             // gutter_padding
        )
}
