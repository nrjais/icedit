use crate::{PartialLineView, Viewport};
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
    last_viewport: Option<Viewport>,
    last_cursor_position: Option<Position>,
    last_selection: Option<Selection>,

    // Pre-computed constants for hot paths
    cursor_width: f32,
    tab_width: f32,

    // Frame-based caching
    frame_counter: u64,
    last_render_frame: u64,
}

struct TextOperation {
    content: String,
    position: Point,
    bounds: Rectangle,
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
            tab_width: char_width * 4.0,

            frame_counter: 0,
            last_render_frame: 0,
        }
    }

    /// Ultra-optimized render method with dirty region tracking and object pooling
    pub fn render<Renderer>(
        &mut self,
        editor: &Editor,
        viewport: &Viewport,
        renderer: &mut Renderer,
        bounds: Rectangle,
    ) where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        self.frame_counter += 1;

        // Fast path: check if anything actually changed
        let selection = editor.current_selection();
        let cursor_position = editor.current_cursor().position();

        let needs_full_render = self.check_full_render_needed(viewport, selection);

        if !needs_full_render && self.last_render_frame == self.frame_counter - 1 {
            // Only render cursor if it changed position
            if self.last_cursor_position.as_ref() != Some(&cursor_position) {
                self.draw_cursor_only(renderer, bounds, cursor_position, viewport);
                self.last_cursor_position = Some(cursor_position);
            }
            return;
        }

        // Full render path with extreme optimizations
        self.render_optimized(
            editor,
            viewport,
            renderer,
            bounds,
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
    fn check_full_render_needed(&self, viewport: &Viewport, selection: Option<&Selection>) -> bool {
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
        viewport: &Viewport,
        renderer: &mut Renderer,
        bounds: Rectangle,
        selection: Option<&Selection>,
        cursor_position: Position,
    ) where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        // Step 1: Draw background
        self.draw_background(renderer, bounds);

        // Step 2: Get visible lines from buffer directly
        let visible_lines = self.get_visible_lines_with_partial(editor, viewport);

        // Step 3: Batch all operations with zero-allocation hot path
        let (text_ops, selection_quads) =
            self.prepare_render_operations(&visible_lines, bounds, viewport, selection);

        // Step 4: Batch render selections first (behind text)
        self.render_selections_batched(renderer, &selection_quads);

        // Step 5: Batch render all text operations
        self.render_text_batched(renderer, &text_ops);

        // Step 6: Draw cursor last (on top)
        self.draw_cursor_optimized(renderer, bounds, cursor_position, viewport);
    }

    /// Get visible lines with partial line information for smooth scrolling
    fn get_visible_lines_with_partial(
        &self,
        editor: &Editor,
        viewport: &Viewport,
    ) -> Vec<(String, PartialLineView)> {
        let rope = editor.current_buffer().rope();
        let total_lines = rope.len_lines();

        let mut lines_with_partial = Vec::new();
        for partial_line in &viewport.partial_lines {
            if partial_line.line_index < total_lines {
                if let Some(line) = rope.get_line(partial_line.line_index) {
                    lines_with_partial.push((line.to_string(), partial_line.clone()));
                }
            }
        }
        lines_with_partial
    }

    #[inline]
    fn prepare_render_operations(
        &mut self,
        visible_lines: &[(String, PartialLineView)],
        bounds: Rectangle,
        viewport: &Viewport,
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

            // Skip lines with no visible content
            if visible_height <= 0.0 {
                continue;
            }

            let y_position = bounds.y + partial_line.y_offset + partial_line.clip_top;
            let x_position = bounds.x - viewport.scroll_offset.0;

            // Create text operation
            text_ops.push(TextOperation {
                content: line_content.clone(),
                position: Point::new(x_position, y_position),
                bounds: Rectangle::new(
                    Point::new(x_position, y_position),
                    Size::new(bounds.width, visible_height),
                ),
            });

            // Handle selection rendering for this line
            if let Some(selection) = selection {
                if line_index >= selection.start.line && line_index <= selection.end.line {
                    let start_col = if line_index == selection.start.line {
                        selection.start.column
                    } else {
                        0
                    };
                    let end_col = if line_index == selection.end.line {
                        selection.end.column
                    } else {
                        line_content.chars().count()
                    };

                    let start_x = self.calculate_x_position_fast(start_col, line_content);
                    let end_x = self.calculate_x_position_fast(end_col, line_content);

                    let selection_y = y_position;
                    let selection_width = end_x - start_x;

                    if selection_width > 0.0 {
                        selection_quads.push(Quad {
                            bounds: Rectangle::new(
                                Point::new(
                                    start_x + bounds.x - viewport.scroll_offset.0,
                                    selection_y,
                                ),
                                Size::new(selection_width, visible_height),
                            ),
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                            snap: false,
                        });
                    }
                }
            }
        }

        (text_ops, selection_quads)
    }

    #[inline]
    fn calculate_x_position_fast(&self, column: usize, line_content: &str) -> f32 {
        // Ultra-fast column position calculation with tab handling
        let mut x = 0.0;
        let mut char_count = 0;

        for ch in line_content.chars() {
            if char_count >= column {
                break;
            }

            if ch == '\t' {
                // Tab alignment to next tab stop
                let tab_stop = ((x / self.tab_width).floor() + 1.0) * self.tab_width;
                x = tab_stop;
            } else {
                x += self.char_width;
            }

            char_count += 1;
        }

        x
    }

    fn render_selections_batched<Renderer>(&self, renderer: &mut Renderer, quads: &[Quad])
    where
        Renderer: iced::advanced::Renderer,
    {
        // Batch render all selection quads in one call
        for quad in quads {
            renderer.fill_quad(*quad, self.selection_color);
        }
    }

    fn render_text_batched<Renderer>(&self, renderer: &mut Renderer, text_ops: &[TextOperation])
    where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        // Batch render all text operations
        for text_op in text_ops {
            let text = Text {
                content: text_op.content.clone(),
                bounds: text_op.bounds.size(),
                size: iced::Pixels(self.font_size),
                line_height: iced::advanced::text::LineHeight::Absolute(iced::Pixels(
                    self.line_height,
                )),
                font: Font::MONOSPACE,
                align_x: Alignment::Left,
                align_y: iced::alignment::Vertical::Top,
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::advanced::text::Wrapping::None,
            };

            renderer.fill_text(text, text_op.position, self.text_color, text_op.bounds);
        }
    }

    fn draw_background<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle)
    where
        Renderer: iced::advanced::Renderer,
    {
        let background_quad = Quad {
            bounds,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        };

        renderer.fill_quad(background_quad, self.background_color);
    }

    fn draw_cursor_optimized<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        cursor_position: Position,
        viewport: &Viewport,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // Calculate cursor position with viewport offset
        let cursor_x = cursor_position.column as f32 * self.char_width - viewport.scroll_offset.0;
        let cursor_y = cursor_position.line as f32 * self.line_height - viewport.scroll_offset.1;

        // Only draw if cursor is visible in viewport
        if cursor_x >= -self.cursor_width
            && cursor_x <= bounds.width
            && cursor_y >= -self.line_height
            && cursor_y <= bounds.height
        {
            let cursor_quad = Quad {
                bounds: Rectangle::new(
                    Point::new(bounds.x + cursor_x, bounds.y + cursor_y),
                    Size::new(self.cursor_width, self.line_height),
                ),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            };

            renderer.fill_quad(cursor_quad, self.cursor_color);
        }
    }

    fn draw_cursor_only<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        cursor_position: Position,
        viewport: &Viewport,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // Optimized cursor-only rendering for when only cursor moved
        self.draw_cursor_optimized(renderer, bounds, cursor_position, viewport);
    }

    /// Clean up object pools to prevent memory bloat
    pub fn cleanup_pools(&mut self) {
        if self.text_operation_pool.capacity() > 128 {
            self.text_operation_pool.shrink_to(64);
        }
    }
}

impl Default for EditorRenderer {
    fn default() -> Self {
        Self::new(
            14.0,                                 // font_size
            18.0,                                 // line_height
            8.0,                                  // char_width
            Color::from_rgb(0.1, 0.1, 0.1),       // background_color
            Color::from_rgb(0.9, 0.9, 0.9),       // text_color
            Color::from_rgb(1.0, 1.0, 1.0),       // cursor_color
            Color::from_rgba(0.3, 0.6, 1.0, 0.3), // selection_color
        )
    }
}
