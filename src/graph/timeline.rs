/// Time axis management for graph view
/// Handles scrolling, zooming, and time range selection

#[derive(Debug, Clone)]
pub struct Timeline {
    /// Visible time window start (seconds)
    pub view_start: f64,
    /// Visible time window end (seconds)
    pub view_end: f64,
    /// Total data time range start
    pub data_start: f64,
    /// Total data time range end
    pub data_end: f64,
    /// Auto-scroll (follow latest data)
    pub auto_scroll: bool,
    /// Time window width for auto-scroll (seconds)
    pub auto_scroll_window: f64,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            view_start: 0.0,
            view_end: 10.0,
            data_start: 0.0,
            data_end: 0.0,
            auto_scroll: true,
            auto_scroll_window: 10.0,
        }
    }

    /// Update the data range
    pub fn update_data_range(&mut self, start: f64, end: f64) {
        self.data_start = start;
        self.data_end = end;

        if self.auto_scroll {
            self.view_end = end;
            self.view_start = (end - self.auto_scroll_window).max(self.data_start);
        }
    }

    /// Zoom in/out centered on a time point
    pub fn zoom(&mut self, center_time: f64, factor: f64) {
        let left = center_time - self.view_start;
        let right = self.view_end - center_time;
        self.view_start = center_time - left * factor;
        self.view_end = center_time + right * factor;

        // Clamp minimum window
        if self.view_end - self.view_start < 0.001 {
            self.view_start = center_time - 0.0005;
            self.view_end = center_time + 0.0005;
        }

        self.auto_scroll = false;
    }

    /// Scroll by a time delta
    pub fn scroll(&mut self, delta_time: f64) {
        self.view_start += delta_time;
        self.view_end += delta_time;
        self.auto_scroll = false;
    }

    /// Set the view range
    pub fn set_view_range(&mut self, start: f64, end: f64) {
        self.view_start = start;
        self.view_end = end;
        self.auto_scroll = false;
    }

    /// View width in seconds
    pub fn view_width(&self) -> f64 {
        self.view_end - self.view_start
    }

    /// Fit all data in view
    pub fn fit_all(&mut self) {
        if self.data_end > self.data_start {
            let margin = (self.data_end - self.data_start) * 0.05;
            self.view_start = self.data_start - margin;
            self.view_end = self.data_end + margin;
        }
        self.auto_scroll = false;
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}
