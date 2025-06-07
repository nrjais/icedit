use crate::{utils, PartialLineView, Viewport};
use iced::{
    advanced::{
        renderer::Quad,
        text::{Alignment, Text},
    },
    Color, Font, Point, Rectangle, Size,
};
use icedit_core::{Editor, Position, Selection};
use std::collections::VecDeque;

/// Information about visible columns in a line for horizontal scrolling optimization
#[derive(Debug, Clone, Copy)]
pub struct PartialColumnView {
    /// Starting column index (inclusive)
    pub start_column: usize,
    /// Ending column index (exclusive)
    pub end_column: usize,
    /// X offset from the start of the visible content
    pub x_offset: f32,
    /// Width of the visible portion
    pub visible_width: f32,
}

impl PartialColumnView {
    pub fn new(start_column: usize, end_column: usize, x_offset: f32, visible_width: f32) -> Self {
        Self {
            start_column,
            end_column,
            x_offset,
            visible_width,
        }
    }
}

/// Scrollbar information for rendering
#[derive(Debug, Clone, Copy)]
pub struct ScrollbarInfo {
    /// Whether this scrollbar should be visible
    pub visible: bool,
    /// Track bounds (background of scrollbar)
    pub track_bounds: Rectangle,
    /// Thumb bounds (draggable part)
    pub thumb_bounds: Rectangle,
    /// Scroll ratio (0.0 to 1.0)
    pub scroll_ratio: f32,
}

impl Default for ScrollbarInfo {
    fn default() -> Self {
        Self {
            visible: false,
            track_bounds: Rectangle::default(),
            thumb_bounds: Rectangle::default(),
            scroll_ratio: 0.0,
        }
    }
}

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

    // Gutter properties
    gutter_width: f32,
    gutter_background_color: Color,
    line_number_color: Color,
    current_line_number_color: Color,
    gutter_padding: f32,

    // Scrollbar properties
    scrollbar_width: f32,
    scrollbar_track_color: Color,
    scrollbar_thumb_color: Color,
    min_scrollbar_thumb_size: f32,

    // Optimization caches and pools
    text_operation_pool: VecDeque<TextOperation>,
    last_viewport: Option<Viewport>,
    last_cursor_position: Option<Position>,
    last_selection: Option<Selection>,

    // Scrollbar state caching
    last_vertical_scrollbar: Option<ScrollbarInfo>,
    last_horizontal_scrollbar: Option<ScrollbarInfo>,
    last_content_dimensions: Option<(f32, f32)>, // (width, height)

    // Pre-computed constants for hot paths
    cursor_width: f32,
    tab_width: f32,

    // Frame-based caching
    frame_counter: u64,
    last_render_frame: u64,

    // Content width caching for optimization
    cached_max_line_width: Option<f32>,
    cached_max_line_index: Option<usize>,
    content_width_dirty: bool,
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
        gutter_width: f32,
        gutter_background_color: Color,
        line_number_color: Color,
        current_line_number_color: Color,
        gutter_padding: f32,
    ) -> Self {
        Self {
            font_size,
            line_height,
            char_width,
            background_color,
            text_color,
            cursor_color,
            selection_color,

            // Gutter properties
            gutter_width,
            gutter_background_color,
            line_number_color,
            current_line_number_color,
            gutter_padding,

            // Scrollbar styling
            scrollbar_width: 12.0,
            scrollbar_track_color: Color::from_rgba(0.5, 0.5, 0.5, 0.2),
            scrollbar_thumb_color: Color::from_rgba(0.6, 0.6, 0.6, 0.8),
            min_scrollbar_thumb_size: 16.0,

            // Initialize pools with reasonable capacity
            text_operation_pool: VecDeque::with_capacity(64),
            last_viewport: None,
            last_cursor_position: None,
            last_selection: None,

            // Scrollbar cache
            last_vertical_scrollbar: None,
            last_horizontal_scrollbar: None,
            last_content_dimensions: None,

            // Pre-compute constants
            cursor_width: 2.0,
            tab_width: char_width * 4.0,

            frame_counter: 0,
            last_render_frame: 0,

            // Content width caching for optimization
            cached_max_line_width: None,
            cached_max_line_index: None,
            content_width_dirty: true,
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
                self.draw_cursor_only(renderer, bounds, cursor_position, viewport, editor);
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
        // Calculate content dimensions for scrollbar visibility
        let content_dimensions = self.calculate_content_dimensions(editor);

        // Calculate scrollbar info (lazy - only if needed)
        let (vertical_scrollbar, horizontal_scrollbar) =
            self.calculate_scrollbars(viewport, bounds, content_dimensions);

        // Adjust editor content bounds to account for scrollbars
        let editor_bounds = self.calculate_editor_content_bounds(
            bounds,
            vertical_scrollbar.visible,
            horizontal_scrollbar.visible,
        );

        // Step 1: Draw background
        self.draw_background(renderer, bounds);

        // Step 2: Draw gutter if enabled
        self.draw_gutter(renderer, bounds, editor, viewport, cursor_position);

        // Step 3: Get visible lines from buffer directly
        let visible_lines = self.get_visible_lines_with_partial(editor, viewport);

        // Step 4: Batch all operations with zero-allocation hot path
        let (text_ops, selection_quads) =
            self.prepare_render_operations(&visible_lines, editor_bounds, viewport, selection);

        // Step 5: Batch render selections first (behind text)
        self.render_selections_batched(renderer, &selection_quads);

        // Step 6: Batch render all text operations
        self.render_text_batched(renderer, &text_ops);

        // Step 7: Draw cursor (on top of text)
        self.draw_cursor_optimized(renderer, editor_bounds, cursor_position, viewport, editor);

        // Step 8: Draw scrollbars last (on top of everything)
        self.render_scrollbars(renderer, vertical_scrollbar, horizontal_scrollbar);

        // Update scrollbar cache
        self.last_vertical_scrollbar = Some(vertical_scrollbar);
        self.last_horizontal_scrollbar = Some(horizontal_scrollbar);
        self.last_content_dimensions = Some(content_dimensions);
    }

    /// Calculate the total content dimensions for scrollbar calculations with caching
    fn calculate_content_dimensions(&mut self, editor: &Editor) -> (f32, f32) {
        let rope = editor.current_buffer().rope();
        let line_count = rope.len_lines();

        // Calculate content height
        let content_height = line_count as f32 * self.line_height;

        // Use cached content width if available and not dirty
        let content_width = if self.content_width_dirty || self.cached_max_line_width.is_none() {
            // Use the common utility function with increased limit for better accuracy
            let max_width = utils::calculate_max_content_width(editor, self.char_width, 2000);

            // Cache the result (without padding since the utility already adds it)
            let max_width_without_padding = max_width - self.char_width * 2.0;
            self.cached_max_line_width = Some(max_width_without_padding);
            self.cached_max_line_index = Some(0); // We don't track line index in the utility
            self.content_width_dirty = false;

            max_width
        } else {
            // Use cached value with padding
            self.cached_max_line_width.unwrap() + self.char_width * 2.0
        };

        (content_width, content_height)
    }

    /// Calculate the width of a line accounting for tabs
    fn calculate_line_width(&self, line: &str) -> f32 {
        utils::calculate_line_width(line, self.char_width, self.tab_width)
    }

    /// Calculate scrollbar visibility and positions (lazy evaluation)
    fn calculate_scrollbars(
        &mut self,
        viewport: &Viewport,
        bounds: Rectangle,
        content_dimensions: (f32, f32),
    ) -> (ScrollbarInfo, ScrollbarInfo) {
        let (content_width, content_height) = content_dimensions;

        // Check if we can use cached values
        if let (Some(last_content), Some(last_v_scroll), Some(last_h_scroll)) = (
            self.last_content_dimensions,
            self.last_vertical_scrollbar,
            self.last_horizontal_scrollbar,
        ) {
            // Use cache if content dimensions and viewport haven't changed significantly
            if (last_content.0 - content_width).abs() < 1.0
                && (last_content.1 - content_height).abs() < 1.0
                && self.last_viewport.as_ref().map_or(false, |v| {
                    (v.size.0 - viewport.size.0).abs() < 1.0
                        && (v.size.1 - viewport.size.1).abs() < 1.0
                })
            {
                return (last_v_scroll, last_h_scroll);
            }
        }

        // Calculate vertical scrollbar
        let vertical_scrollbar = if content_height > viewport.size.1 {
            let track_height = bounds.height
                - if content_width > viewport.size.0 {
                    self.scrollbar_width
                } else {
                    0.0
                };

            let thumb_height = f32::max(
                viewport.size.1 / content_height * track_height,
                self.min_scrollbar_thumb_size,
            );

            let scroll_range = content_height - viewport.size.1;
            let scroll_ratio = if scroll_range > 0.0 {
                viewport.scroll_offset.1 / scroll_range
            } else {
                0.0
            };

            let thumb_y = scroll_ratio * (track_height - thumb_height);

            ScrollbarInfo {
                visible: true,
                track_bounds: Rectangle::new(
                    Point::new(bounds.x + bounds.width - self.scrollbar_width, bounds.y),
                    Size::new(self.scrollbar_width, track_height),
                ),
                thumb_bounds: Rectangle::new(
                    Point::new(
                        bounds.x + bounds.width - self.scrollbar_width,
                        bounds.y + thumb_y,
                    ),
                    Size::new(self.scrollbar_width, thumb_height),
                ),
                scroll_ratio,
            }
        } else {
            ScrollbarInfo::default()
        };

        // Calculate horizontal scrollbar
        let horizontal_scrollbar = if content_width > viewport.size.0 {
            let track_width = bounds.width
                - if vertical_scrollbar.visible {
                    self.scrollbar_width
                } else {
                    0.0
                };

            let thumb_width = f32::max(
                viewport.size.0 / content_width * track_width,
                self.min_scrollbar_thumb_size,
            );

            let scroll_range = content_width - viewport.size.0;
            let scroll_ratio = if scroll_range > 0.0 {
                viewport.scroll_offset.0 / scroll_range
            } else {
                0.0
            };

            let thumb_x = scroll_ratio * (track_width - thumb_width);

            ScrollbarInfo {
                visible: true,
                track_bounds: Rectangle::new(
                    Point::new(bounds.x, bounds.y + bounds.height - self.scrollbar_width),
                    Size::new(track_width, self.scrollbar_width),
                ),
                thumb_bounds: Rectangle::new(
                    Point::new(
                        bounds.x + thumb_x,
                        bounds.y + bounds.height - self.scrollbar_width,
                    ),
                    Size::new(thumb_width, self.scrollbar_width),
                ),
                scroll_ratio,
            }
        } else {
            ScrollbarInfo::default()
        };

        (vertical_scrollbar, horizontal_scrollbar)
    }

    /// Calculate the bounds for editor content, accounting for scrollbars and gutter
    fn calculate_editor_content_bounds(
        &self,
        bounds: Rectangle,
        vertical_scrollbar_visible: bool,
        horizontal_scrollbar_visible: bool,
    ) -> Rectangle {
        let width_reduction = if vertical_scrollbar_visible {
            self.scrollbar_width
        } else {
            0.0
        };
        let height_reduction = if horizontal_scrollbar_visible {
            self.scrollbar_width
        } else {
            0.0
        };

        // Always account for gutter width since it's always enabled, plus 4px padding
        let gutter_offset = self.gutter_width + 4.0;

        Rectangle::new(
            Point::new(bounds.x + gutter_offset, bounds.y),
            Size::new(
                bounds.width - width_reduction - gutter_offset,
                bounds.height - height_reduction,
            ),
        )
    }

    /// Render both scrollbars
    fn render_scrollbars<Renderer>(
        &self,
        renderer: &mut Renderer,
        vertical_scrollbar: ScrollbarInfo,
        horizontal_scrollbar: ScrollbarInfo,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // Render vertical scrollbar
        if vertical_scrollbar.visible {
            self.render_single_scrollbar(renderer, vertical_scrollbar);
        }

        // Render horizontal scrollbar
        if horizontal_scrollbar.visible {
            self.render_single_scrollbar(renderer, horizontal_scrollbar);
        }

        // Render corner piece if both scrollbars are visible
        if vertical_scrollbar.visible && horizontal_scrollbar.visible {
            let corner_bounds = Rectangle::new(
                Point::new(
                    vertical_scrollbar.track_bounds.x,
                    horizontal_scrollbar.track_bounds.y,
                ),
                Size::new(self.scrollbar_width, self.scrollbar_width),
            );

            let corner_quad = Quad {
                bounds: corner_bounds,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: false,
            };

            renderer.fill_quad(corner_quad, self.scrollbar_track_color);
        }
    }

    /// Render a single scrollbar (vertical or horizontal)
    fn render_single_scrollbar<Renderer>(&self, renderer: &mut Renderer, scrollbar: ScrollbarInfo)
    where
        Renderer: iced::advanced::Renderer,
    {
        // Render track
        let track_quad = Quad {
            bounds: scrollbar.track_bounds,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        };
        renderer.fill_quad(track_quad, self.scrollbar_track_color);

        // Render thumb
        let thumb_quad = Quad {
            bounds: scrollbar.thumb_bounds,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        };
        renderer.fill_quad(thumb_quad, self.scrollbar_thumb_color);
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

            // Calculate visible column range for horizontal scrolling optimization
            let column_view = self.calculate_visible_columns(
                line_content,
                viewport.scroll_offset.0,
                viewport.size.0,
            );

            // Create text operation
            text_ops.push(TextOperation {
                content: if let Some(col_view) = column_view {
                    // Only render the visible portion of the line
                    self.extract_visible_line_content(line_content, &col_view)
                } else {
                    line_content.clone()
                },
                position: Point::new(
                    x_position + column_view.map_or(0.0, |cv| cv.x_offset),
                    y_position,
                ),
                bounds: Rectangle::new(
                    Point::new(
                        x_position + column_view.map_or(0.0, |cv| cv.x_offset),
                        y_position,
                    ),
                    Size::new(
                        column_view.map_or(bounds.width, |cv| cv.visible_width),
                        visible_height,
                    ),
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
        utils::calculate_column_x_position(column, line_content, self.char_width)
    }

    /// Calculate which columns are visible given horizontal scroll offset and viewport width
    fn calculate_visible_columns(
        &self,
        line_content: &str,
        horizontal_scroll: f32,
        viewport_width: f32,
    ) -> Option<PartialColumnView> {
        let line_width = self.calculate_line_width(line_content);

        // If the entire line fits in the viewport, no clipping needed
        if line_width <= viewport_width && horizontal_scroll <= 0.0 {
            return None;
        }

        // Find start and end columns based on horizontal scroll
        let mut start_column = 0;
        let mut end_column = line_content.chars().count();
        let mut x_offset = 0.0;

        // Find the start column
        let mut current_x = 0.0;
        let mut char_index = 0;
        for ch in line_content.chars() {
            if current_x >= horizontal_scroll {
                start_column = char_index;
                x_offset = current_x - horizontal_scroll;
                break;
            }

            if ch == '\t' {
                let tab_stop = ((current_x / self.tab_width).floor() + 1.0) * self.tab_width;
                current_x = tab_stop;
            } else {
                current_x += self.char_width;
            }
            char_index += 1;
        }

        // Find the end column
        let visible_end = horizontal_scroll + viewport_width;
        current_x = 0.0;
        char_index = 0;
        for ch in line_content.chars() {
            if current_x >= visible_end {
                end_column = char_index;
                break;
            }

            if ch == '\t' {
                let tab_stop = ((current_x / self.tab_width).floor() + 1.0) * self.tab_width;
                current_x = tab_stop;
            } else {
                current_x += self.char_width;
            }
            char_index += 1;
        }

        let visible_width = f32::min(viewport_width, line_width - horizontal_scroll);

        Some(PartialColumnView::new(
            start_column,
            end_column,
            x_offset,
            visible_width,
        ))
    }

    /// Extract only the visible portion of a line based on column view
    fn extract_visible_line_content(
        &self,
        line_content: &str,
        column_view: &PartialColumnView,
    ) -> String {
        let chars: Vec<char> = line_content.chars().collect();
        let start = column_view.start_column.min(chars.len());
        let end = column_view.end_column.min(chars.len());

        if start >= end {
            return String::new();
        }

        chars[start..end].iter().collect()
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
                shaping: iced::advanced::text::Shaping::Advanced,
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
        editor: &Editor,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // Calculate cursor position with proper tab handling
        let cursor_y = cursor_position.line as f32 * self.line_height - viewport.scroll_offset.1;

        // Get the line content to calculate accurate X position with tab handling
        // Note: bounds.x already includes gutter offset from calculate_editor_content_bounds
        let cursor_x = {
            let rope = editor.current_buffer().rope();
            if cursor_position.line < rope.len_lines() {
                if let Some(line) = rope.get_line(cursor_position.line) {
                    let line_str = line.to_string();
                    utils::calculate_column_x_position(
                        cursor_position.column,
                        &line_str,
                        self.char_width,
                    ) - viewport.scroll_offset.0
                } else {
                    cursor_position.column as f32 * self.char_width - viewport.scroll_offset.0
                }
            } else {
                cursor_position.column as f32 * self.char_width - viewport.scroll_offset.0
            }
        };

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
        editor: &Editor,
    ) where
        Renderer: iced::advanced::Renderer,
    {
        // Optimized cursor-only rendering for when only cursor moved
        self.draw_cursor_optimized(renderer, bounds, cursor_position, viewport, editor);
    }

    /// Draw the line number gutter (always enabled)
    fn draw_gutter<Renderer>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        _editor: &Editor,
        viewport: &Viewport,
        cursor_position: Position,
    ) where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = Font>,
    {
        if self.gutter_width <= 0.0 {
            return;
        }

        // Draw gutter background
        let gutter_bounds = Rectangle::new(
            bounds.position(),
            Size::new(self.gutter_width, bounds.height),
        );

        let gutter_quad = Quad {
            bounds: gutter_bounds,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: false,
        };

        renderer.fill_quad(gutter_quad, self.gutter_background_color);

        // Draw line numbers for visible lines using the same logic as text rendering
        for partial_line in &viewport.partial_lines {
            let line_index = partial_line.line_index;
            let line_number = line_index + 1; // Line numbers are 1-based
            let y_position = bounds.y + partial_line.y_offset;

            // Skip if line is not visible
            if y_position + self.line_height < bounds.y || y_position > bounds.y + bounds.height {
                continue;
            }

            // Choose color based on whether this is the current line
            let color = if line_index == cursor_position.line {
                self.current_line_number_color
            } else {
                self.line_number_color
            };

            // Render line number - using simple left-aligned approach first
            let line_number_text = line_number.to_string();
            let text_position = Point::new(bounds.x + self.gutter_padding, y_position);
            let text_bounds = Rectangle::new(
                text_position,
                Size::new(self.gutter_width - self.gutter_padding, self.line_height),
            );

            let text = iced::advanced::text::Text {
                content: line_number_text,
                bounds: text_bounds.size(),
                size: iced::Pixels(self.font_size),
                line_height: iced::advanced::text::LineHeight::Absolute(iced::Pixels(
                    self.line_height,
                )),
                font: Font::MONOSPACE,
                align_x: iced::advanced::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Top,
                shaping: iced::advanced::text::Shaping::Basic,
                wrapping: iced::advanced::text::Wrapping::None,
            };

            renderer.fill_text(text, text_position, color, text_bounds);
        }
    }

    /// Clean up object pools to prevent memory bloat
    pub fn cleanup_pools(&mut self) {
        if self.text_operation_pool.capacity() > 128 {
            self.text_operation_pool.shrink_to(64);
        }
    }

    /// Invalidate content width cache when editor content changes
    pub fn invalidate_content_cache(&mut self) {
        self.content_width_dirty = true;
        self.cached_max_line_width = None;
        self.cached_max_line_index = None;
    }

    /// Get the maximum content width for limiting horizontal scrolling
    pub fn get_max_content_width(&self) -> Option<f32> {
        self.cached_max_line_width
            .map(|w| w + self.char_width * 2.0)
    }
}
