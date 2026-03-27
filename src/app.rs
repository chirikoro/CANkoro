/// Main application state and orchestration
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};

use crate::can::frame::CanFrame;
use crate::can::receiver::CanReceiver;
use crate::can::transmitter::{CanTransmitter, ForwardConfig, TrapezoidalConfig, TxCommand};
use crate::config::tx_settings::TxSettings;
use crate::dbc::database::DbcDatabase;
use crate::log::asc::AscWriter;
use crate::log::blf::BlfWriter;
use crate::ui::channel_config::ChannelConfigState;
use crate::ui::graph_view::{GraphMode, GraphViewState};
use crate::ui::log_view::LogViewState;
use crate::ui::main_view::{self, MainTab};
use crate::ui::panel_editor::PanelEditorState;
use crate::ui::tx_config::{self, TxAction};
use crate::ui::tx_panel;
use crate::vector::channel::{CanMode, CanPort, ChannelConfig};
use crate::vector::driver::VectorDriver;

pub struct CankoroApp {
    // Tab state
    current_tab: MainTab,

    // Channel configuration
    channel_config: ChannelConfigState,

    // CAN runtime
    driver: Option<VectorDriver>,
    port: Option<Arc<Mutex<CanPort>>>,
    receiver: Option<CanReceiver>,
    transmitter: Option<CanTransmitter>,
    rx_channel: Option<Receiver<CanFrame>>,
    tx_forward_sender: Option<Sender<CanFrame>>,

    // DBC
    dbc: Arc<RwLock<Option<DbcDatabase>>>,

    // UI state
    log_view: LogViewState,
    graph_view: GraphViewState,
    tx_settings: TxSettings,
    panel_editor: PanelEditorState,

    // Connection state
    connected: bool,
    rx_count: u64,
    tx_count: u64,

    // Logging
    logging: bool,
    asc_writer: Option<AscWriter>,
    blf_writer: Option<BlfWriter>,

    // Error messages
    error_message: Option<String>,

    // Available channel info
    available_channels: Vec<(u32, String)>,
}

impl CankoroApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut channel_config = ChannelConfigState::new();

        // Try to open Vector driver
        let driver = match VectorDriver::new() {
            Ok(drv) => {
                if let Ok(channels) = drv.get_channels() {
                    channel_config.set_channels(channels);
                }
                Some(drv)
            }
            Err(e) => {
                log::warn!("Vector driver not available: {}", e);
                None
            }
        };

        let available_channels = channel_config
            .channels
            .iter()
            .map(|ch| (ch.hw_info.channel_index, ch.hw_info.name.clone()))
            .collect();

        Self {
            current_tab: MainTab::ChannelConfig,
            channel_config,
            driver,
            port: None,
            receiver: None,
            transmitter: None,
            rx_channel: None,
            tx_forward_sender: None,
            dbc: Arc::new(RwLock::new(None)),
            log_view: LogViewState::new(),
            graph_view: GraphViewState::new(),
            tx_settings: TxSettings::new(),
            panel_editor: PanelEditorState::new(),
            connected: false,
            rx_count: 0,
            tx_count: 0,
            logging: false,
            asc_writer: None,
            blf_writer: None,
            error_message: None,
            available_channels,
        }
    }

    fn connect(&mut self) {
        let driver = match &self.driver {
            Some(d) => d,
            None => {
                self.error_message = Some("Vector driver not available".to_string());
                return;
            }
        };

        // Build channel configs
        let mut configs = Vec::new();
        for ch in &self.channel_config.channels {
            if ch.enabled {
                configs.push(ChannelConfig {
                    channel_mask: ch.hw_info.channel_mask,
                    mode: ch.mode,
                    bitrate: ch.bitrate,
                    data_bitrate: ch.data_bitrate,
                });
            }
        }

        if configs.is_empty() {
            self.error_message = Some("No channels enabled".to_string());
            return;
        }

        // Open port
        match CanPort::open(driver.api().clone(), "CANkoro", &configs) {
            Ok(mut port) => {
                if let Err(e) = port.activate() {
                    self.error_message = Some(format!("Activate failed: {}", e));
                    return;
                }

                let port = Arc::new(Mutex::new(port));

                // Start receiver
                let (rx_tx, rx_rx) = crossbeam_channel::bounded(65536);
                let receiver = CanReceiver::start(
                    port.clone(),
                    rx_tx,
                    Duration::from_millis(1),
                );

                // Start transmitter
                let (fwd_tx, fwd_rx) = crossbeam_channel::bounded(65536);
                let transmitter = CanTransmitter::start(
                    port.clone(),
                    fwd_rx,
                    self.dbc.clone(),
                );

                self.port = Some(port);
                self.receiver = Some(receiver);
                self.transmitter = Some(transmitter);
                self.rx_channel = Some(rx_rx);
                self.tx_forward_sender = Some(fwd_tx);
                self.connected = true;
                self.error_message = None;

                log::info!("Connected to CAN interface");
            }
            Err(e) => {
                self.error_message = Some(format!("Open port failed: {}", e));
            }
        }
    }

    fn disconnect(&mut self) {
        self.stop_logging();
        if let Some(mut tx) = self.transmitter.take() {
            tx.stop();
        }
        if let Some(mut rx) = self.receiver.take() {
            rx.stop();
        }
        self.port = None;
        self.rx_channel = None;
        self.tx_forward_sender = None;
        self.connected = false;
        log::info!("Disconnected from CAN interface");
    }

    fn start_logging(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ASC files", &["asc"])
            .add_filter("BLF files", &["blf"])
            .save_file()
        {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("asc")
                .to_lowercase();

            match ext.as_str() {
                "blf" => match BlfWriter::new(&path) {
                    Ok(writer) => {
                        self.blf_writer = Some(writer);
                        self.logging = true;
                    }
                    Err(e) => self.error_message = Some(format!("BLF open error: {}", e)),
                },
                _ => match AscWriter::new(&path) {
                    Ok(writer) => {
                        self.asc_writer = Some(writer);
                        self.logging = true;
                    }
                    Err(e) => self.error_message = Some(format!("ASC open error: {}", e)),
                },
            }
        }
    }

    fn stop_logging(&mut self) {
        if let Some(mut writer) = self.asc_writer.take() {
            let _ = writer.flush();
        }
        if let Some(mut writer) = self.blf_writer.take() {
            let _ = writer.finalize();
        }
        self.logging = false;
    }

    fn load_dbc(&mut self, path: &str) {
        match DbcDatabase::load(Path::new(path)) {
            Ok(db) => {
                log::info!(
                    "Loaded DBC: {} messages",
                    db.messages().len()
                );
                *self.dbc.write() = Some(db);
            }
            Err(e) => {
                self.error_message = Some(format!("DBC load error: {}", e));
            }
        }
    }

    fn process_received_frames(&mut self) {
        if let Some(ref rx) = self.rx_channel {
            let mut new_frames = Vec::new();
            // Drain all available frames
            while let Ok(frame) = rx.try_recv() {
                new_frames.push(frame);
            }

            if !new_frames.is_empty() {
                self.rx_count += new_frames.len() as u64;

                // Forward to transmitter for forwarding mode
                if let Some(ref fwd_tx) = self.tx_forward_sender {
                    for frame in &new_frames {
                        let _ = fwd_tx.try_send(frame.clone());
                    }
                }

                // Update graph view
                if let Some(ref dbc_db) = *self.dbc.read() {
                    for frame in &new_frames {
                        self.graph_view
                            .add_frame_data(frame.timestamp, frame.id, &frame.data, dbc_db);
                    }
                }

                // Write to log
                if self.logging {
                    for frame in &new_frames {
                        if let Some(ref mut writer) = self.asc_writer {
                            let _ = writer.write_frame(frame);
                        }
                        if let Some(ref mut writer) = self.blf_writer {
                            let _ = writer.write_frame(frame);
                        }
                    }
                }

                // Update log view
                if !self.log_view.paused {
                    self.log_view.add_frames(new_frames);
                }
            }
        }
    }

    fn process_playback(&mut self, dt: f64) {
        if self.graph_view.mode != GraphMode::Playback || !self.graph_view.playback.playing {
            return;
        }

        if self.graph_view.playback.loaded_frames.is_empty() {
            return;
        }

        let speed = self.graph_view.playback.speed;
        self.graph_view.playback.current_time += dt * speed;

        let start_time = self.graph_view.playback.loaded_frames[0].timestamp;
        let target_time = start_time + self.graph_view.playback.current_time;

        // Collect frames to process (clone to avoid borrow conflict)
        let mut frames_to_process = Vec::new();
        while self.graph_view.playback.current_index < self.graph_view.playback.loaded_frames.len()
        {
            let frame = &self.graph_view.playback.loaded_frames
                [self.graph_view.playback.current_index];
            if frame.timestamp <= target_time {
                frames_to_process.push(frame.clone());
                self.graph_view.playback.current_index += 1;
            } else {
                break;
            }
        }

        // Now process collected frames
        if let Some(ref dbc_db) = *self.dbc.read() {
            for frame in &frames_to_process {
                self.graph_view.add_frame_data(
                    frame.timestamp,
                    frame.id,
                    &frame.data,
                    dbc_db,
                );
            }
        }
        if !self.log_view.paused {
            for frame in frames_to_process {
                self.log_view.add_frame(frame);
            }
        }

        // Check if playback is done
        if self.graph_view.playback.current_index
            >= self.graph_view.playback.loaded_frames.len()
        {
            self.graph_view.playback.playing = false;
        }
    }

    fn handle_tx_actions(&mut self, actions: Vec<TxAction>) {
        for action in actions {
            match action {
                TxAction::StartForward(idx) => {
                    if let Some(ref tx) = self.transmitter {
                        let fwd = &self.tx_settings.forwards[idx];
                        let ch_configs = &self.channel_config.channels;
                        let rx_mask = ch_configs
                            .get(fwd.rx_channel_index as usize)
                            .map(|c| c.hw_info.channel_mask)
                            .unwrap_or(0);
                        let tx_mask = ch_configs
                            .get(fwd.tx_channel_index as usize)
                            .map(|c| c.hw_info.channel_mask)
                            .unwrap_or(0);
                        tx.send_command(TxCommand::StartForward(ForwardConfig {
                            rx_channel: fwd.rx_channel_index as u8,
                            rx_channel_mask: rx_mask,
                            tx_channel_mask: tx_mask,
                            message_filter: fwd.message_filter.clone(),
                            modifications: fwd.modifications.clone(),
                        }));
                    }
                }
                TxAction::StopForward(_) => {
                    if let Some(ref tx) = self.transmitter {
                        tx.send_command(TxCommand::StopForward);
                    }
                }
                TxAction::StartTrapezoidal(idx) => {
                    if let Some(ref tx) = self.transmitter {
                        let trap = &self.tx_settings.trapezoidals[idx];
                        let ch_configs = &self.channel_config.channels;
                        let tx_mask = ch_configs
                            .get(trap.tx_channel_index as usize)
                            .map(|c| c.hw_info.channel_mask)
                            .unwrap_or(0);
                        tx.send_command(TxCommand::StartTrapezoidal(TrapezoidalConfig {
                            tx_channel_mask: tx_mask,
                            message_id: trap.message_id,
                            signal_name: trap.signal_name.clone(),
                            initial_value: trap.initial_value,
                            max_value: trap.max_value,
                            rate: trap.rate,
                            hold_time_ms: trap.hold_time_ms,
                            cycle_time_ms: trap.cycle_time_ms,
                            repeat: trap.repeat,
                            base_data: trap.base_data.clone(),
                            is_fd: trap.is_fd,
                            dlc: trap.dlc,
                        }));
                    }
                }
                TxAction::StopTrapezoidal(_) => {
                    if let Some(ref tx) = self.transmitter {
                        tx.send_command(TxCommand::StopTrapezoidal);
                    }
                }
            }
        }
    }
}

impl eframe::App for CankoroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process CAN frames from receive thread
        self.process_received_frames();

        // Process playback
        let dt = ctx.input(|i| i.predicted_dt as f64);
        self.process_playback(dt);

        // Request continuous repaint for real-time updates
        if self.connected || self.graph_view.playback.playing {
            ctx.request_repaint();
        }

        // Top menu bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("CANkoro");
                ui.separator();

                if self.connected {
                    if ui.button("Disconnect").clicked() {
                        self.disconnect();
                    }
                } else {
                    if ui.button("Connect").clicked() {
                        // Load DBC if specified
                        if let Some(ref path) = self.channel_config.selected_dbc_path.clone() {
                            self.load_dbc(path);
                        }
                        self.connect();
                    }
                }

                ui.separator();

                if self.connected {
                    if self.logging {
                        if ui.button("Stop Logging").clicked() {
                            self.stop_logging();
                        }
                    } else {
                        if ui.button("Start Logging").clicked() {
                            self.start_logging();
                        }
                    }
                }

                // Error display
                if let Some(ref err) = self.error_message {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, err);
                }
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            main_view::draw_status_bar(ui, self.connected, self.rx_count, self.tx_count, self.logging);
        });

        // Tab bar
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            main_view::draw_tab_bar(ui, &mut self.current_tab);
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                MainTab::ChannelConfig => {
                    let changed = crate::ui::channel_config::draw_channel_config(
                        ui,
                        &mut self.channel_config,
                    );
                    if changed {
                        self.available_channels = self
                            .channel_config
                            .channels
                            .iter()
                            .map(|ch| (ch.hw_info.channel_index, ch.hw_info.name.clone()))
                            .collect();
                    }
                }
                MainTab::LogView => {
                    crate::ui::log_view::draw_log_view(ui, &mut self.log_view);
                }
                MainTab::GraphView => {
                    let dbc = self.dbc.read().clone();
                    crate::ui::graph_view::draw_graph_view(
                        ui,
                        &mut self.graph_view,
                        &dbc,
                    );
                }
                MainTab::TxConfig => {
                    let dbc = self.dbc.read().clone();
                    let actions = tx_config::draw_tx_config(
                        ui,
                        &mut self.tx_settings,
                        &dbc,
                        &self.available_channels,
                    );
                    self.handle_tx_actions(actions);
                }
                MainTab::TxPanels => {
                    let dbc = self.dbc.read().clone();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for panel in &mut self.tx_settings.panels {
                            let _events = tx_panel::draw_tx_panel(ui, panel, &dbc);
                            // Handle panel events (signal changes)
                        }
                    });
                }
                MainTab::PanelEditor => {
                    let dbc = self.dbc.read().clone();
                    crate::ui::panel_editor::draw_panel_editor(
                        ui,
                        &mut self.tx_settings.panels,
                        &mut self.panel_editor,
                        &dbc,
                    );
                }
            }
        });
    }
}
