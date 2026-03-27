/// CAN log display table
use egui::{self, Ui};

use crate::can::frame::{CanFrame, Direction};

/// State for the log view
pub struct LogViewState {
    /// All received frames (ring buffer)
    pub frames: Vec<CanFrame>,
    /// Maximum number of frames to keep
    pub max_frames: usize,
    /// Auto-scroll to bottom
    pub auto_scroll: bool,
    /// Filter by channel
    pub channel_filter: Option<u8>,
    /// Filter by message ID
    pub id_filter: String,
    /// Pause display
    pub paused: bool,
    /// Frame counter
    pub total_count: u64,
}

impl LogViewState {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(10000),
            max_frames: 50000,
            auto_scroll: true,
            channel_filter: None,
            id_filter: String::new(),
            paused: false,
            total_count: 0,
        }
    }

    pub fn add_frame(&mut self, frame: CanFrame) {
        self.total_count += 1;
        if self.frames.len() >= self.max_frames {
            self.frames.remove(0);
        }
        self.frames.push(frame);
    }

    pub fn add_frames(&mut self, frames: Vec<CanFrame>) {
        for frame in frames {
            self.add_frame(frame);
        }
    }

    pub fn clear(&mut self) {
        self.frames.clear();
        self.total_count = 0;
    }
}

impl Default for LogViewState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_log_view(ui: &mut Ui, state: &mut LogViewState) {
    ui.horizontal(|ui| {
        ui.heading("CAN Log");
        ui.separator();
        ui.label(format!("Total: {}", state.total_count));
        ui.label(format!("Displayed: {}", state.frames.len()));
        ui.separator();
        ui.checkbox(&mut state.auto_scroll, "Auto-scroll");
        if ui.button(if state.paused { "Resume" } else { "Pause" }).clicked() {
            state.paused = !state.paused;
        }
        if ui.button("Clear").clicked() {
            state.clear();
        }
        ui.separator();
        ui.label("ID Filter:");
        ui.text_edit_singleline(&mut state.id_filter);
    });

    ui.separator();

    let id_filter_val = if state.id_filter.is_empty() {
        None
    } else {
        u32::from_str_radix(state.id_filter.trim_start_matches("0x"), 16).ok()
    };

    let row_height = 18.0;
    let num_rows = state.frames.len();

    // Table header
    ui.horizontal(|ui| {
        ui.set_min_width(ui.available_width());
        let widths = [90.0, 30.0, 25.0, 80.0, 30.0, 25.0, 30.0, 300.0];
        let headers = ["Timestamp", "Ch", "Dir", "ID", "DLC", "FD", "BRS", "Data"];
        for (w, h) in widths.iter().zip(headers.iter()) {
            ui.allocate_ui(egui::vec2(*w, row_height), |ui| {
                ui.strong(*h);
            });
        }
    });

    ui.separator();

    // Scrollable table body
    let scroll = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(ui.available_height());

    scroll.show_rows(ui, row_height, num_rows, |ui, row_range| {
        for i in row_range {
            let frame = &state.frames[i];

            // Apply filters
            if let Some(ch_filter) = state.channel_filter {
                if frame.channel != ch_filter {
                    continue;
                }
            }
            if let Some(id_f) = id_filter_val {
                if frame.id != id_f {
                    continue;
                }
            }

            let dir_color = match frame.direction {
                Direction::Rx => egui::Color32::from_rgb(100, 200, 100),
                Direction::Tx => egui::Color32::from_rgb(100, 150, 255),
            };

            ui.horizontal(|ui| {
                ui.set_min_width(ui.available_width());
                ui.allocate_ui(egui::vec2(90.0, row_height), |ui| {
                    ui.monospace(format!("{:.6}", frame.timestamp));
                });
                ui.allocate_ui(egui::vec2(30.0, row_height), |ui| {
                    ui.monospace(format!("{}", frame.channel + 1));
                });
                ui.allocate_ui(egui::vec2(25.0, row_height), |ui| {
                    let dir_str = match frame.direction {
                        Direction::Rx => "Rx",
                        Direction::Tx => "Tx",
                    };
                    ui.colored_label(dir_color, dir_str);
                });
                ui.allocate_ui(egui::vec2(80.0, row_height), |ui| {
                    ui.monospace(frame.id_hex());
                });
                ui.allocate_ui(egui::vec2(30.0, row_height), |ui| {
                    ui.monospace(format!("{}", frame.dlc));
                });
                ui.allocate_ui(egui::vec2(25.0, row_height), |ui| {
                    ui.monospace(if frame.is_fd { "Y" } else { "N" });
                });
                ui.allocate_ui(egui::vec2(30.0, row_height), |ui| {
                    ui.monospace(if frame.is_brs { "Y" } else { "-" });
                });
                ui.allocate_ui(egui::vec2(300.0, row_height), |ui| {
                    ui.monospace(frame.data_hex());
                });
            });
        }
    });
}
