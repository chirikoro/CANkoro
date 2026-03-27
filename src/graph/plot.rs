/// Signal plot rendering using egui_plot
use egui::Color32;

/// Data for a single signal plot
#[derive(Debug, Clone)]
pub struct SignalPlotData {
    /// Signal name
    pub name: String,
    /// Message ID
    pub message_id: u32,
    /// Message name
    pub message_name: String,
    /// Unit
    pub unit: String,
    /// (timestamp, physical_value) pairs
    pub points: Vec<(f64, f64)>,
    /// Plot color
    pub color: Color32,
    /// Whether this signal is visible
    pub visible: bool,
    /// Y-axis range (min, max) from DBC
    pub y_min: f64,
    pub y_max: f64,
}

impl SignalPlotData {
    pub fn new(
        name: String,
        message_id: u32,
        message_name: String,
        unit: String,
        y_min: f64,
        y_max: f64,
        color: Color32,
    ) -> Self {
        Self {
            name,
            message_id,
            message_name,
            unit,
            points: Vec::new(),
            color,
            visible: true,
            y_min,
            y_max,
        }
    }

    /// Add a data point
    pub fn add_point(&mut self, timestamp: f64, value: f64) {
        self.points.push((timestamp, value));
    }

    /// Downsample points for display performance
    /// Uses largest-triangle-three-buckets algorithm
    pub fn downsample(&self, max_points: usize) -> Vec<[f64; 2]> {
        if self.points.len() <= max_points {
            return self.points.iter().map(|&(t, v)| [t, v]).collect();
        }

        let n = self.points.len();
        let bucket_size = (n as f64) / (max_points as f64);
        let mut result = Vec::with_capacity(max_points);

        // Always include first point
        result.push([self.points[0].0, self.points[0].1]);

        for i in 1..max_points - 1 {
            let bucket_start = ((i as f64) * bucket_size) as usize;
            let bucket_end = (((i + 1) as f64) * bucket_size) as usize;
            let bucket_end = bucket_end.min(n);

            // Next bucket average for triangle calculation
            let next_start = bucket_end;
            let next_end = (((i + 2) as f64) * bucket_size) as usize;
            let next_end = next_end.min(n);

            let mut avg_x = 0.0;
            let mut avg_y = 0.0;
            let next_count = next_end - next_start;
            if next_count > 0 {
                for j in next_start..next_end {
                    avg_x += self.points[j].0;
                    avg_y += self.points[j].1;
                }
                avg_x /= next_count as f64;
                avg_y /= next_count as f64;
            }

            // Find point in current bucket that forms largest triangle
            let prev = result.last().unwrap();
            let mut max_area = 0.0;
            let mut max_idx = bucket_start;

            for j in bucket_start..bucket_end {
                let area = ((prev[0] - avg_x) * (self.points[j].1 - prev[1])
                    - (prev[0] - self.points[j].0) * (avg_y - prev[1]))
                    .abs();
                if area > max_area {
                    max_area = area;
                    max_idx = j;
                }
            }

            result.push([self.points[max_idx].0, self.points[max_idx].1]);
        }

        // Always include last point
        if let Some(last) = self.points.last() {
            result.push([last.0, last.1]);
        }

        result
    }

    /// Clear all data points
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Trim points older than the given time
    pub fn trim_before(&mut self, time: f64) {
        self.points.retain(|&(t, _)| t >= time);
    }
}

/// Color palette for signal plots
pub const PLOT_COLORS: [Color32; 16] = [
    Color32::from_rgb(31, 119, 180),   // blue
    Color32::from_rgb(255, 127, 14),   // orange
    Color32::from_rgb(44, 160, 44),    // green
    Color32::from_rgb(214, 39, 40),    // red
    Color32::from_rgb(148, 103, 189),  // purple
    Color32::from_rgb(140, 86, 75),    // brown
    Color32::from_rgb(227, 119, 194),  // pink
    Color32::from_rgb(127, 127, 127),  // gray
    Color32::from_rgb(188, 189, 34),   // olive
    Color32::from_rgb(23, 190, 207),   // cyan
    Color32::from_rgb(255, 187, 120),  // light orange
    Color32::from_rgb(152, 223, 138),  // light green
    Color32::from_rgb(255, 152, 150),  // light red
    Color32::from_rgb(197, 176, 213),  // light purple
    Color32::from_rgb(196, 156, 148),  // light brown
    Color32::from_rgb(247, 182, 210),  // light pink
];

pub fn get_plot_color(index: usize) -> Color32 {
    PLOT_COLORS[index % PLOT_COLORS.len()]
}
