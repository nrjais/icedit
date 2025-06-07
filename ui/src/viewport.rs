/// Information about a partially visible line for smooth scrolling
///
/// This struct contains information about lines that are partially visible
/// in the viewport, enabling smooth scrolling by rendering only the visible
/// portions of each line with proper clipping.
#[derive(Debug, Clone)]
pub struct PartialLineView {
    /// Line index in the buffer
    pub line_index: usize,
    /// Y position relative to viewport top (can be negative for partially visible lines)
    pub y_offset: f32,
    /// How much of the line is visible (0.0 to 1.0)
    pub visible_fraction: f32,
    /// The clipped area of the line that should be rendered
    pub clip_top: f32,
    pub clip_bottom: f32,
}

/// Viewport information for rendering optimization with partial line support
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Scroll offset (x, y) - supports fractional positions for smooth scrolling
    pub scroll_offset: (f32, f32),
    /// Viewport size (width, height)
    pub size: (f32, f32),
    /// Character dimensions for text layout
    pub char_width: f32,
    pub line_height: f32,
    /// Visible line range (start_line, end_line) - calculated based on scroll and viewport
    pub visible_lines: (usize, usize),
    /// Information about partially visible lines at top and bottom
    pub partial_lines: Vec<PartialLineView>,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            scroll_offset: (0.0, 0.0),
            size: (800.0, 600.0),
            char_width: 8.0,
            line_height: 18.0,
            visible_lines: (0, 0),
            partial_lines: Vec::new(),
        }
    }

    /// Update viewport size and recalculate visible lines
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size = (width, height);
        self.update_visible_lines();
    }

    /// Update scroll offset and recalculate visible lines
    pub fn set_scroll_offset(&mut self, x: f32, y: f32) {
        self.scroll_offset = (x, y);
        self.update_visible_lines();
    }

    /// Set character dimensions
    pub fn set_char_dimensions(&mut self, char_width: f32, line_height: f32) {
        self.char_width = char_width;
        self.line_height = line_height;
        self.update_visible_lines();
    }

    /// Calculate which lines are visible based on scroll and viewport, including partial lines
    fn update_visible_lines(&mut self) {
        let scroll_y = self.scroll_offset.1;
        let viewport_height = self.size.1;
        let line_height = self.line_height;

        // Calculate the range of lines that intersect with the viewport
        let first_line_top = (scroll_y / line_height).floor();
        let last_line_bottom = ((scroll_y + viewport_height) / line_height).ceil();

        let start_line = first_line_top as usize;
        let end_line = last_line_bottom as usize;

        self.visible_lines = (start_line, end_line);

        // Calculate partial line information
        self.partial_lines.clear();

        for line_idx in start_line..end_line {
            let line_y_top = line_idx as f32 * line_height;
            let line_y_bottom = line_y_top + line_height;
            let viewport_top = scroll_y;
            let viewport_bottom = scroll_y + viewport_height;

            // Check if line intersects with viewport
            if line_y_bottom > viewport_top && line_y_top < viewport_bottom {
                let y_offset = line_y_top - viewport_top;

                // Calculate clipping
                let clip_top = if line_y_top < viewport_top {
                    viewport_top - line_y_top
                } else {
                    0.0
                };

                let clip_bottom = if line_y_bottom > viewport_bottom {
                    line_y_bottom - viewport_bottom
                } else {
                    0.0
                };

                let visible_height = line_height - clip_top - clip_bottom;
                let visible_fraction = visible_height / line_height;

                // Only consider lines that have some visible portion
                if visible_fraction > 0.0 {
                    self.partial_lines.push(PartialLineView {
                        line_index: line_idx,
                        y_offset,
                        visible_fraction,
                        clip_top,
                        clip_bottom,
                    });
                }
            }
        }
    }

    /// Check if a line is currently visible
    pub fn is_line_visible(&self, line: usize) -> bool {
        line >= self.visible_lines.0 && line <= self.visible_lines.1
    }

    /// Check if a position (line, column) is currently visible in the viewport
    pub fn is_position_visible(&self, line: usize, column: usize, char_width: f32) -> bool {
        // Check vertical visibility
        if !self.is_line_visible(line) {
            return false;
        }

        // Check horizontal visibility (approximate)
        let line_y = line as f32 * self.line_height;
        let column_x = column as f32 * char_width; // Simplified, doesn't handle tabs

        let viewport_left = self.scroll_offset.0;
        let viewport_right = self.scroll_offset.0 + self.size.0;
        let viewport_top = self.scroll_offset.1;
        let viewport_bottom = self.scroll_offset.1 + self.size.1;

        line_y >= viewport_top
            && line_y + self.line_height <= viewport_bottom
            && column_x >= viewport_left
            && column_x + char_width <= viewport_right
    }

    /// Get scroll bounds to prevent over-scrolling
    pub fn clamp_scroll_offset(&self, offset: (f32, f32), content_lines: usize) -> (f32, f32) {
        let (x, y) = offset;
        let content_height = content_lines as f32 * self.line_height;

        let clamped_x = x.max(0.0);
        let clamped_y = if content_height > self.size.1 {
            y.max(0.0).min(content_height - self.size.1)
        } else {
            0.0
        };

        (clamped_x, clamped_y)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
