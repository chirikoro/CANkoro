/// Graph view UI with signal plotting, cursors, and playback
use egui::{self, Color32, Stroke, Ui};
use egui_plot::{Line, Plot, PlotPoints, VLine};

use crate::dbc::database::DbcDatabase;
use crate::graph::cursor::CursorManager;
use crate::graph::plot::{get_plot_color, SignalPlotData};
use crate::graph::timeline::Timeline;

/// Graph display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphMode {
    /// Real-time display (live data)
    Realtime,
    /// Log file playback
    Playback,
}

/// State for the graph view
pub struct GraphViewState {
    /// Signal plot data
    pub signals: Vec<SignalPlotData>,
    /// Timeline controller
    pub timeline: Timeline,
    /// Cursor manager
    pub cursors: CursorManager,
    /// Display mode
    pub mode: GraphMode,
    /// Playback state
    pub playback: PlaybackState,
    /// Max points to display per signal
    pub max_display_points: usize,
    /// Signal selection dialog open
    pub signal_select_open: bool,
    /// Selected signals for display (message_id, signal_name)
    pub selected_signals: Vec<(u32, String)>,
}

/// Playback state for log file mode
pub struct PlaybackState {
    pub loaded_frames: Vec<crate::can::frame::CanFrame>,
    pub current_index: usize,
    pub playing: bool,
    pub speed: f64,
    pub current_time: f64,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            loaded_frames: Vec::new(),
            current_index: 0,
            playing: false,
            speed: 1.0,
            current_time: 0.0,
        }
    }
}

impl GraphViewState {
    pub fn new() -> Self {
        Self {
            signals: Vec::new(),
            timeline: Timeline::new(),
            cursors: CursorManager::new(),
            mode: GraphMode::Realtime,
            playback: PlaybackState::default(),
            max_display_points: 2000,
            signal_select_open: false,
            selected_signals: Vec::new(),
        }
    }

    /// Add signal data from a received frame
    pub fn add_frame_data(
        &mut self,
        timestamp: f64,
        message_id: u32,
        data: &[u8],
        dbc: &DbcDatabase,
    ) {
        if let Some(msg) = dbc.get_message(message_id) {
            for sig in &msg.signals {
                // Check if this signal is selected for display
                if !self
                    .selected_signals
                    .iter()
                    .any(|(mid, sname)| *mid == message_id && sname == &sig.name)
                {
                    continue;
                }

                let value = sig.decode_physical(data);

                // Find or create plot data for this signal
                let plot = self.signals.iter_mut().find(|s| {
                    s.message_id == message_id && s.name == sig.name
                });

                if let Some(plot) = plot {
                    plot.add_point(timestamp, value);
                } else {
                    let color = get_plot_color(self.signals.len());
                    let mut plot_data = SignalPlotData::new(
                        sig.name.clone(),
                        message_id,
                        msg.name.clone(),
                        sig.unit.clone(),
                        sig.min,
                        sig.max,
                        color,
                    );
                    plot_data.add_point(timestamp, value);
                    self.signals.push(plot_data);
                }
            }

            // Update timeline
            self.timeline.update_data_range(
                self.timeline.data_start.min(timestamp),
                timestamp,
            );
        }
    }

    pub fn clear_data(&mut self) {
        for sig in &mut self.signals {
            sig.clear();
        }
        self.timeline = Timeline::new();
    }
}

impl Default for GraphViewState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_graph_view(
    ui: &mut Ui,
    state: &mut GraphViewState,
    dbc: &Option<DbcDatabase>,
) {
    // Toolbar
    ui.horizontal(|ui| {
        ui.heading("Signal Graph");
        ui.separator();

        // Mode selection
        ui.radio_value(&mut state.mode, GraphMode::Realtime, "Realtime");
        ui.radio_value(&mut state.mode, GraphMode::Playback, "Playback");

        ui.separator();

        if ui.button("Select Signals").clicked() {
            state.signal_select_open = true;
        }

        if ui.button("Fit All").clicked() {
            state.timeline.fit_all();
        }

        ui.checkbox(&mut state.timeline.auto_scroll, "Auto-scroll");

        ui.separator();

        // Cursor controls
        ui.checkbox(&mut state.cursors.cursor1.visible, "C1");
        ui.checkbox(&mut state.cursors.diff_mode, "Diff");
        if state.cursors.diff_mode {
            ui.checkbox(&mut state.cursors.cursor2.visible, "C2");
        }

        if ui.button("Clear Data").clicked() {
            state.clear_data();
        }
    });

    // Playback controls
    if state.mode == GraphMode::Playback {
        ui.horizontal(|ui| {
            if ui.button("Load Log").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Log files", &["asc", "blf"])
                    .pick_file()
                {
                    match crate::log::reader::load_log_file(&path) {
                        Ok(frames) => {
                            state.playback.loaded_frames = frames;
                            state.playback.current_index = 0;
                            state.playback.current_time = 0.0;
                            state.clear_data();
                            log::info!("Loaded {} frames from log", state.playback.loaded_frames.len());
                        }
                        Err(e) => {
                            log::error!("Failed to load log: {}", e);
                        }
                    }
                }
            }

            let play_text = if state.playback.playing { "Pause" } else { "Play" };
            if ui.button(play_text).clicked() {
                state.playback.playing = !state.playback.playing;
            }

            if ui.button("Stop").clicked() {
                state.playback.playing = false;
                state.playback.current_index = 0;
                state.playback.current_time = 0.0;
                state.clear_data();
            }

            ui.label("Speed:");
            ui.add(egui::Slider::new(&mut state.playback.speed, 0.1..=10.0).suffix("x"));

            ui.label(format!(
                "Frame: {}/{}",
                state.playback.current_index,
                state.playback.loaded_frames.len()
            ));
        });
    }

    ui.separator();

    // Signal selection window
    if state.signal_select_open {
        draw_signal_selector(ui, state, dbc);
    }

    // Draw plots - each signal gets its own plot with independent Y axis
    let available_height = ui.available_height();
    let num_visible = state.signals.iter().filter(|s| s.visible).count().max(1);
    let plot_height = (available_height / num_visible as f32).max(80.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let view_start = state.timeline.view_start;
            let view_end = state.timeline.view_end;
            let cursor1_time = state.cursors.cursor1.time;
            let cursor2_time = state.cursors.cursor2.time;
            let c1_visible = state.cursors.cursor1.visible;
            let c2_visible = state.cursors.diff_mode && state.cursors.cursor2.visible;

            for (idx, signal) in state.signals.iter().enumerate() {
                if !signal.visible {
                    continue;
                }

                ui.group(|ui| {
                    // Signal header
                    ui.horizontal(|ui| {
                        ui.colored_label(signal.color, &signal.name);
                        ui.label(format!(
                            "({} - 0x{:03X}) [{}]",
                            signal.message_name, signal.message_id, signal.unit
                        ));

                        // Show cursor values
                        if c1_visible {
                            if let Some(v) = CursorManager::interpolate_value(
                                cursor1_time,
                                &signal.points,
                            ) {
                                ui.label(format!("C1: {:.3} {}", v, signal.unit));
                            }
                        }
                        if c2_visible {
                            if let Some(v) = CursorManager::interpolate_value(
                                cursor2_time,
                                &signal.points,
                            ) {
                                ui.label(format!("C2: {:.3} {}", v, signal.unit));
                            }
                        }
                        if c1_visible && c2_visible {
                            let v1 = CursorManager::interpolate_value(
                                cursor1_time,
                                &signal.points,
                            );
                            let v2 = CursorManager::interpolate_value(
                                cursor2_time,
                                &signal.points,
                            );
                            if let (Some(v1), Some(v2)) = (v1, v2) {
                                ui.colored_label(
                                    Color32::YELLOW,
                                    format!(
                                        "dV: {:.3} {} | dt: {:.6}s",
                                        v2 - v1,
                                        signal.unit,
                                        cursor2_time - cursor1_time
                                    ),
                                );
                            }
                        }
                    });

                    // Plot
                    let downsampled = signal.downsample(state.max_display_points);
                    let points: PlotPoints = downsampled.into();
                    let line = Line::new(points)
                        .color(signal.color)
                        .stroke(Stroke::new(1.5, signal.color))
                        .name(&signal.name);

                    let plot_id = format!("signal_plot_{}", idx);
                    let mut plot = Plot::new(&plot_id)
                        .height(plot_height)
                        .allow_scroll(false)
                        .allow_zoom(egui::Vec2b::new(true, false))
                        .allow_drag(egui::Vec2b::new(true, false))
                        .x_axis_label("Time [s]")
                        .y_axis_label(format!("{} [{}]", signal.name, signal.unit))
                        .include_y(signal.y_min)
                        .include_y(signal.y_max)
                        .link_axis("time_axis", egui::Vec2b::new(true, false))
                        .link_cursor("time_axis", egui::Vec2b::new(true, false));

                    if !state.timeline.auto_scroll {
                        plot = plot
                            .include_x(view_start)
                            .include_x(view_end);
                    }

                    plot.show(ui, |plot_ui| {
                        plot_ui.line(line);

                        // Draw cursors
                        if c1_visible {
                            plot_ui.vline(
                                VLine::new(cursor1_time)
                                    .color(Color32::from_rgb(255, 255, 0))
                                    .width(1.5)
                                    .name("C1"),
                            );
                        }
                        if c2_visible {
                            plot_ui.vline(
                                VLine::new(cursor2_time)
                                    .color(Color32::from_rgb(0, 255, 255))
                                    .width(1.5)
                                    .name("C2"),
                            );
                        }

                        // Cursor interaction is handled via the linked cursor group
                    });
                });
            }

            // Cursor info panel at bottom
            if c1_visible || c2_visible {
                ui.separator();
                draw_cursor_info(ui, state);
            }
        });
}

fn draw_cursor_info(ui: &mut Ui, state: &GraphViewState) {
    ui.group(|ui| {
        ui.heading("Cursor Analysis");
        ui.horizontal(|ui| {
            if state.cursors.cursor1.visible {
                ui.label(format!("C1: {:.6} s", state.cursors.cursor1.time));
            }
            if state.cursors.diff_mode && state.cursors.cursor2.visible {
                ui.label(format!("C2: {:.6} s", state.cursors.cursor2.time));
                ui.label(format!("dt: {:.6} s", state.cursors.delta_time()));
                if let Some(freq) = state.cursors.frequency() {
                    ui.label(format!("freq: {:.2} Hz", freq));
                }
            }
        });

        // Table of cursor values per signal
        egui::Grid::new("cursor_values_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Signal");
                if state.cursors.cursor1.visible {
                    ui.strong("C1 Value");
                }
                if state.cursors.diff_mode && state.cursors.cursor2.visible {
                    ui.strong("C2 Value");
                    ui.strong("Delta");
                }
                ui.strong("Unit");
                ui.end_row();

                for signal in &state.signals {
                    if !signal.visible {
                        continue;
                    }
                    ui.label(&signal.name);

                    if state.cursors.cursor1.visible {
                        if let Some(v) = CursorManager::interpolate_value(
                            state.cursors.cursor1.time,
                            &signal.points,
                        ) {
                            ui.monospace(format!("{:.4}", v));
                        } else {
                            ui.label("-");
                        }
                    }

                    if state.cursors.diff_mode && state.cursors.cursor2.visible {
                        let v1 = CursorManager::interpolate_value(
                            state.cursors.cursor1.time,
                            &signal.points,
                        );
                        let v2 = CursorManager::interpolate_value(
                            state.cursors.cursor2.time,
                            &signal.points,
                        );
                        if let Some(v2) = v2 {
                            ui.monospace(format!("{:.4}", v2));
                        } else {
                            ui.label("-");
                        }
                        if let (Some(v1), Some(v2)) = (v1, v2) {
                            ui.colored_label(Color32::YELLOW, format!("{:.4}", v2 - v1));
                        } else {
                            ui.label("-");
                        }
                    }

                    ui.label(&signal.unit);
                    ui.end_row();
                }
            });
    });
}

fn draw_signal_selector(
    ui: &mut Ui,
    state: &mut GraphViewState,
    dbc: &Option<DbcDatabase>,
) {
    egui::Window::new("Signal Selection")
        .open(&mut state.signal_select_open)
        .resizable(true)
        .default_size([400.0, 500.0])
        .show(ui.ctx(), |ui| {
            if let Some(ref dbc) = dbc {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for msg in dbc.messages() {
                        ui.collapsing(
                            format!("0x{:03X} - {}", msg.id, msg.name),
                            |ui| {
                                for sig in &msg.signals {
                                    let is_selected = state
                                        .selected_signals
                                        .iter()
                                        .any(|(mid, sname)| {
                                            *mid == msg.id && sname == &sig.name
                                        });
                                    let mut selected = is_selected;
                                    ui.horizontal(|ui| {
                                        if ui.checkbox(&mut selected, &sig.name).changed() {
                                            if selected {
                                                state.selected_signals.push((
                                                    msg.id,
                                                    sig.name.clone(),
                                                ));
                                            } else {
                                                state.selected_signals.retain(
                                                    |(mid, sname)| {
                                                        !(*mid == msg.id
                                                            && sname == &sig.name)
                                                    },
                                                );
                                                // Remove plot data
                                                state.signals.retain(|s| {
                                                    !(s.message_id == msg.id
                                                        && s.name == sig.name)
                                                });
                                            }
                                        }
                                        ui.label(format!(
                                            "[{:.1}..{:.1}] {}",
                                            sig.min, sig.max, sig.unit
                                        ));
                                    });
                                }
                            },
                        );
                    }
                });
            } else {
                ui.label("No DBC file loaded. Please load a DBC file first.");
            }
        });
}
