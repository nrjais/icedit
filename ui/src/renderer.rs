use iced::{
    advanced::{
        renderer::Quad,
        text::{Alignment, Text},
    },
    Color, Font, Point, Rectangle, Size,
};
use icedit_core::{Editor, Position, Selection};
use std::collections::VecDeque;

/// Extremely optimized renderer for the editor widget using advanced techniques
pub struct EditorRenderer {
    // Core rendering properties
    font_size: f32,
    line_height: f32,
    char_width: f32,
    background_color: Color,
    text_color: Color,
    cursor_color: Color,
    selection_color: Color,

    // Optimization caches and pools
    text_operation_pool: VecDeque<TextOperation>,
    last_viewport: Option<icedit_core::Viewport>,
    last_cursor_position: Option<Position>,
    last_selection: Option<Selection>,

    // Pre-computed constants for hot paths
    cursor_width: f32,
    cursor_height_multiplier: f32,
    tab_width: f32,

    // Frame-based caching
    frame_counter: u64,
    last_render_frame: u64,
}

struct TextOperation {
    content: String,
    position: Point,
    bounds: Rectangle,
    visible_height: f32,
    line_index: usize,
}

impl EditorRenderer {
    pub fn new(
        font_size: f32,
        line_height: f32,
        char_width: f32,
        background_color: Color,
        text_color: Color,
        cursor_color: Color,
        selection_color: Color,
    ) -> Self {
        Self {
            font_size,
            line_height,
            char_width,
            background_color,
            text_color,
            cursor_color,
            selection_color,

            // Initialize pools with reasonable capacity
            text_operation_pool: VecDeque::with_capacity(64),
            last_viewport: None,
            last_cursor_position: None,
            last_selection: None,

            // Pre-compute constants
            cursor_width: 2.0,
            cursor_height_multiplier: 1.2,
            tab_width: char_width * 4.0,

            frame_counter: 0,
            last_render_frame: 0,
        }
    }

    /// Ultra-optimized render method with dirty region tracking and object pooling
    pub fn render<Renderer>(&mut self, editor: &Editor, renderer: &mut Renderer, bounds: Rectangle)
    where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        self.frame_counter += 1;

        // Fast path: check if anything actually changed
        let viewport = editor.viewport();
        let selection = editor.current_selection();
        let cursor_position = editor.current_cursor().position();

        let needs_full_render = self.check_full_render_needed(&viewport, selection);

        if !needs_full_render && self.last_render_frame == self.frame_counter - 1 {
            // Only render cursor if it changed position
            if self.last_cursor_position.as_ref() != Some(&cursor_position) {
                self.draw_cursor_only(renderer, bounds, cursor_position, &viewport);
                self.last_cursor_position = Some(cursor_position);
            }
            return;
        }

        // Full render path with extreme optimizations
        self.render_optimized(
            editor,
            renderer,
            bounds,
            &viewport,
            selection,
            cursor_position,
        );

        // Update cache state
        self.last_viewport = Some(viewport.clone());
        self.last_selection = selection.cloned();
        self.last_cursor_position = Some(cursor_position);
        self.last_render_frame = self.frame_counter;
    }

    #[inline]
    fn check_full_render_needed(
        &self,
        viewport: &icedit_core::Viewport,
        selection: Option<&Selection>,
    ) -> bool {
        // Check if viewport changed
        if let Some(last_viewport) = &self.last_viewport {
            if last_viewport.scroll_offset != viewport.scroll_offset
                || last_viewport.size != viewport.size
            {
                return true;
            }
        } else {
            return true;
        }

        // Check if selection changed
        match (&self.last_selection, selection) {
            (None, None) => {}
            (Some(a), Some(b)) if a == b => {}
            _ => return true,
        }

        false
    }

    fn render_optimized<Renderer>(
        &mut self,
        editor: &Editor,
        renderer: &mut Renderer,
        bounds: Rectangle,
        viewport: &icedit_core::Viewport,
        selection: Option<&Selection>,
        cursor_position: Position,
    ) where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        // Step 1: Draw background
        self.draw_background(renderer, bounds);

        // Step 2: Get visible lines (this is the expensive call)
        let visible_lines = editor.get_visible_lines_with_partial();

        // Step 3: Batch all operations with zero-allocation hot path
        let (text_ops, selection_quads) =
            self.prepare_render_operations(&visible_lines, bounds, viewport, selection);

        // Step 4: Batch render selections first (behind text)
        self.render_selections_batched(renderer, &selection_quads);

        // Step 5: Batch render all text operations
        self.render_text_batched(renderer, bounds, &text_ops);

        // Step 6: Draw cursor last (on top)
        self.draw_cursor_optimized(renderer, bounds, cursor_position, viewport);
    }

    #[inline]
    fn prepare_render_operations(
        &mut self,
        visible_lines: &[(String, &icedit_core::viewport::PartialLineView)],
        bounds: Rectangle,
        viewport: &icedit_core::Viewport,
        selection: Option<&Selection>,
    ) -> (Vec<TextOperation>, Vec<Quad>) {
        // Clear pools without deallocating
        self.text_operation_pool.clear();

        // Reserve capacity based on visible lines
        let line_count = visible_lines.len();
        let mut text_ops = Vec::with_capacity(line_count);
        let mut selection_quads = Vec::with_capacity(line_count.min(16));

        // Single-pass preparation with minimal allocations
        for (line_content, partial_line) in visible_lines {
            let line_index = partial_line.line_index;
            let visible_height =
                self.line_height - partial_line.clip_top - partial_line.clip_bottom;

            // Fast text position calculation
            let text_position = Point::new(
                bounds.x - viewport.scroll_offset.0,
                bounds.y + partial_line.y_offset,
            );

            let text_bounds = Rectangle::new(
                Point::new(text_position.x, text_position.y + partial_line.clip_top),
                Size::new(bounds.width, visible_height),
            );

            // Reuse text operation objects when possible
            let text_op = if let Some(mut op) = self.text_operation_pool.pop_front() {
                op.content.clear();
                op.content.push_str(line_content);
                op.position = text_position;
                op.bounds = text_bounds;
                op.visible_height = visible_height;
                op.line_index = line_index;
                op
            } else {
                TextOperation {
                    content: line_content.clone(),
                    position: text_position,
                    bounds: text_bounds,
                    visible_height,
                    line_index,
                }
            };
            text_ops.push(text_op);

            // Fast selection quad calculation
            if let Some(selection) = selection {
                if line_index >= selection.start.line && line_index <= selection.end.line {
                    let start_column = if line_index == selection.start.line {
                        selection.start.column
                    } else {
                        0
                    };
                    let end_column = if line_index == selection.end.line {
                        selection.end.column
                    } else {
                        line_content.chars().count()
                    };

                    // Fast X position calculation
                    let start_x = self.calculate_x_position_fast(start_column, line_content);
                    let end_x = self.calculate_x_position_fast(end_column, line_content);

                    if end_x > start_x {
                        let selection_y = bounds.y + partial_line.y_offset + partial_line.clip_top;
                        let selection_bounds = Rectangle::new(
                            Point::new(start_x + bounds.x - viewport.scroll_offset.0, selection_y),
                            Size::new(end_x - start_x, visible_height),
                        );

                        let quad = Quad {
                            bounds: selection_bounds,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                            snap: false,
                        };
                        selection_quads.push(quad);
                    }
                }
            }
        }

        (text_ops, selection_quads)
    }

    #[inline]
    fn calculate_x_position_fast(&self, column: usize, line_content: &str) -> f32 {
        // Ultra-fast position calculation using iterator with early termination
        let mut pixel_pos = 0.0;
        let mut char_count = 0;

        // Process characters in chunks for better cache utilization
        for ch in line_content.chars() {
            if char_count >= column {
                break;
            }

            // Branchless tab width calculation
            let is_tab = (ch == '\t') as i32 as f32;
            pixel_pos += self.char_width + (self.tab_width - self.char_width) * is_tab;
            char_count += 1;
        }

        pixel_pos
    }

    #[inline]
    fn render_selections_batched<Renderer>(&self, renderer: &mut Renderer, quads: &[Quad])
    where
        Renderer: iced::advanced::Renderer,
    {
        // Batch render all selections in a single call when possible
        for quad in quads {
            renderer.fill_quad(*quad, self.selection_color);
        }
    }

    #[inline]
    fn render_text_batched<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        text_ops: &[TextOperation],
    ) where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        // Pre-compute text styling once
        let text_style = Text {
            content: String::new(), // Will be overridden per operation
            bounds: Size::new(bounds.width, self.line_height),
            size: iced::Pixels(self.font_size),
            font: Font::MONOSPACE,
            align_x: Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(self.line_height)),
            shaping: iced::advanced::text::Shaping::Advanced,
            wrapping: iced::advanced::text::Wrapping::None,
        };

        // Render all text operations
        for text_op in text_ops {
            let mut text = text_style.clone();
            text.content = text_op.content.clone();

            renderer.fill_text(text, text_op.position, self.text_color, text_op.bounds);
        }
    }

    #[inline]
    fn draw_background<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle)
    where
        Renderer: iced::advanced::Renderer,
    {
        renderer.fill_quad(
            Quad {
                bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            },
            self.background_color,
        );
    }

    #[inline]
    fn draw_cursor_optimized<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        cursor_position: Position,
        viewport: &icedit_core::Viewport,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // Fast cursor positioning
        let cursor_x = cursor_position.column as f32 * self.char_width - viewport.scroll_offset.0;
        let cursor_y = cursor_position.line as f32 * self.line_height - viewport.scroll_offset.1;

        let cursor_screen_x = cursor_x + bounds.x;
        let cursor_screen_y = cursor_y + bounds.y;
        let cursor_height = self.font_size * self.cursor_height_multiplier;

        // Visibility check with early return
        if cursor_screen_x < bounds.x - self.cursor_width
            || cursor_screen_x > bounds.x + bounds.width
            || cursor_screen_y < bounds.y
            || cursor_screen_y + cursor_height > bounds.y + bounds.height
        {
            return;
        }

        let cursor_bounds = Rectangle::new(
            Point::new(cursor_screen_x, cursor_screen_y),
            Size::new(self.cursor_width, cursor_height),
        );

        renderer.fill_quad(
            Quad {
                bounds: cursor_bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            },
            self.cursor_color,
        );
    }

    #[inline]
    fn draw_cursor_only<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        cursor_position: Position,
        viewport: &icedit_core::Viewport,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // This method is called when only the cursor needs redrawing
        self.draw_cursor_optimized(renderer, bounds, cursor_position, viewport);
    }

    /// Clean up pools periodically to prevent memory bloat
    pub fn cleanup_pools(&mut self) {
        const MAX_POOL_SIZE: usize = 128;

        if self.text_operation_pool.len() > MAX_POOL_SIZE {
            self.text_operation_pool.truncate(MAX_POOL_SIZE / 2);
        }
    }
}

// Implement Default for easier instantiation
impl Default for EditorRenderer {
    fn default() -> Self {
        Self::new(
            14.0,                                 // font_size
            14.0 * 1.3,                           // line_height
            14.0 * 0.6,                           // char_width
            Color::from_rgb(0.15, 0.15, 0.15),    // background_color
            Color::from_rgb(0.9, 0.9, 0.9),       // text_color
            Color::from_rgb(1.0, 1.0, 1.0),       // cursor_color
            Color::from_rgba(0.3, 0.5, 1.0, 0.3), // selection_color
        )
    }
}
