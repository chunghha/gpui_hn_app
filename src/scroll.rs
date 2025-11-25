/// ScrollState manages the vertical scroll position for scrollable content.
/// Based on the pattern from markdown_viewer.
#[derive(Debug, Clone, Copy)]
pub struct ScrollState {
    /// Current vertical scroll position in pixels
    pub scroll_y: f32,
    /// Maximum scroll position (content_height - viewport_height)
    #[allow(dead_code)]
    pub max_scroll_y: f32,
}

impl ScrollState {
    /// Create a new ScrollState with zero scroll position
    pub fn new() -> Self {
        Self {
            scroll_y: 0.0,
            max_scroll_y: 0.0,
        }
    }

    /// Update scroll position, clamping to valid range [0,  max_scroll_y]
    pub fn scroll_by(&mut self, delta: f32) {
        self.scroll_y = (self.scroll_y + delta).max(0.0);
    }

    /// Reset scroll position to top
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.scroll_y = 0.0;
    }

    /// Set the maximum scroll position based on content and viewport heights
    #[allow(dead_code)]
    pub fn set_max_scroll(&mut self, content_height: f32, viewport_height: f32) {
        self.max_scroll_y = (content_height - viewport_height).max(0.0);
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}
