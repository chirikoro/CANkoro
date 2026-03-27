/// Cursor and differential cursor analysis for graph view
use serde::{Deserialize, Serialize};

/// A single cursor (vertical line on the graph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphCursor {
    /// Time position in seconds
    pub time: f64,
    /// Whether this cursor is being dragged
    pub dragging: bool,
    /// Whether this cursor is visible
    pub visible: bool,
    /// Cursor label
    pub label: String,
}

impl GraphCursor {
    pub fn new(label: &str, time: f64) -> Self {
        Self {
            time,
            dragging: false,
            visible: true,
            label: label.to_string(),
        }
    }
}

/// Cursor analysis result for a signal
#[derive(Debug, Clone)]
pub struct CursorSignalValue {
    pub signal_name: String,
    pub value: f64,
    pub unit: String,
}

/// Differential cursor analysis result
#[derive(Debug, Clone)]
pub struct DiffCursorResult {
    pub signal_name: String,
    pub cursor1_value: f64,
    pub cursor2_value: f64,
    pub delta_value: f64,
    pub delta_time: f64,
    pub unit: String,
}

/// Manages cursor state
#[derive(Debug, Clone)]
pub struct CursorManager {
    /// Primary cursor (Cursor 1)
    pub cursor1: GraphCursor,
    /// Secondary cursor for differential analysis (Cursor 2)
    pub cursor2: GraphCursor,
    /// Whether differential cursor mode is active
    pub diff_mode: bool,
}

impl CursorManager {
    pub fn new() -> Self {
        Self {
            cursor1: GraphCursor::new("C1", 0.0),
            cursor2: GraphCursor::new("C2", 0.0),
            diff_mode: false,
        }
    }

    /// Get the time delta between cursors
    pub fn delta_time(&self) -> f64 {
        self.cursor2.time - self.cursor1.time
    }

    /// Calculate frequency from delta time (Hz)
    pub fn frequency(&self) -> Option<f64> {
        let dt = self.delta_time().abs();
        if dt > 0.0 {
            Some(1.0 / dt)
        } else {
            None
        }
    }

    /// Interpolate signal value at a given time from data points
    pub fn interpolate_value(time: f64, data: &[(f64, f64)]) -> Option<f64> {
        if data.is_empty() {
            return None;
        }
        if data.len() == 1 {
            return Some(data[0].1);
        }

        // Find the two points surrounding the time
        let mut prev = &data[0];
        for point in data.iter() {
            if point.0 >= time {
                if point.0 == time {
                    return Some(point.1);
                }
                // Linear interpolation
                if prev.0 == point.0 {
                    return Some(point.1);
                }
                let t = (time - prev.0) / (point.0 - prev.0);
                return Some(prev.1 + t * (point.1 - prev.1));
            }
            prev = point;
        }

        // Time is beyond the last point
        Some(data.last().unwrap().1)
    }

    /// Get differential cursor results for a signal
    pub fn get_diff_result(
        &self,
        signal_name: &str,
        unit: &str,
        data: &[(f64, f64)],
    ) -> Option<DiffCursorResult> {
        let v1 = Self::interpolate_value(self.cursor1.time, data)?;
        let v2 = Self::interpolate_value(self.cursor2.time, data)?;
        Some(DiffCursorResult {
            signal_name: signal_name.to_string(),
            cursor1_value: v1,
            cursor2_value: v2,
            delta_value: v2 - v1,
            delta_time: self.delta_time(),
            unit: unit.to_string(),
        })
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new()
    }
}
