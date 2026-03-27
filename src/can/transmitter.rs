/// CAN transmission logic: forwarding, modified forwarding, and trapezoidal wave
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::can::frame::{CanFrame, Direction};
use crate::dbc::database::DbcDatabase;
use crate::vector::channel::CanPort;
use crate::vector::types::XLaccess;

/// Signal modification specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalModification {
    pub message_id: u32,
    pub signal_name: String,
    /// Physical value to set
    pub value: f64,
}

/// Trapezoidal wave phase
#[derive(Debug, Clone, Copy, PartialEq)]
enum TrapezoidPhase {
    Rising,
    Hold,
    Falling,
    Done,
}

/// Trapezoidal wave configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapezoidalConfig {
    pub tx_channel_mask: XLaccess,
    pub message_id: u32,
    pub signal_name: String,
    /// Initial physical value
    pub initial_value: f64,
    /// Maximum physical value
    pub max_value: f64,
    /// Rate of change (physical value per second)
    pub rate: f64,
    /// Duration to hold at max value (ms)
    pub hold_time_ms: u64,
    /// Transmission cycle time (ms)
    pub cycle_time_ms: u64,
    /// Whether to repeat the waveform
    pub repeat: bool,
    /// Base data for the frame (other signals)
    pub base_data: Vec<u8>,
    /// Whether this is a CAN FD frame
    pub is_fd: bool,
    /// DLC
    pub dlc: u8,
}

/// Forwarding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardConfig {
    pub rx_channel: u8,
    pub rx_channel_mask: XLaccess,
    pub tx_channel_mask: XLaccess,
    /// Optional: filter by message IDs (empty = forward all)
    pub message_filter: Vec<u32>,
    /// Optional signal modifications
    pub modifications: Vec<SignalModification>,
}

/// Commands sent to the transmitter thread
pub enum TxCommand {
    /// Start forwarding
    StartForward(ForwardConfig),
    /// Stop forwarding
    StopForward,
    /// Start trapezoidal wave
    StartTrapezoidal(TrapezoidalConfig),
    /// Stop trapezoidal wave
    StopTrapezoidal,
    /// Send a single frame
    SendOnce {
        channel_mask: XLaccess,
        frame: CanFrame,
    },
    /// Shutdown the transmitter
    Shutdown,
}

/// Manages CAN transmission in a dedicated thread
pub struct CanTransmitter {
    cmd_tx: Sender<TxCommand>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CanTransmitter {
    /// Start the transmitter thread
    pub fn start(
        port: Arc<Mutex<CanPort>>,
        rx_frames: Receiver<CanFrame>,
        dbc: Arc<parking_lot::RwLock<Option<DbcDatabase>>>,
    ) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::Builder::new()
            .name("can-transmitter".to_string())
            .spawn(move || {
                Self::tx_loop(port, rx_frames, cmd_rx, dbc, running_clone);
            })
            .expect("Failed to spawn CAN transmitter thread");

        Self {
            cmd_tx,
            running,
            handle: Some(handle),
        }
    }

    pub fn send_command(&self, cmd: TxCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    fn tx_loop(
        port: Arc<Mutex<CanPort>>,
        rx_frames: Receiver<CanFrame>,
        cmd_rx: Receiver<TxCommand>,
        dbc: Arc<parking_lot::RwLock<Option<DbcDatabase>>>,
        running: Arc<AtomicBool>,
    ) {
        let mut forward_config: Option<ForwardConfig> = None;
        let mut trap_config: Option<TrapezoidalConfig> = None;
        let mut trap_phase = TrapezoidPhase::Rising;
        let mut trap_current_value: f64 = 0.0;
        let mut trap_phase_start = Instant::now();
        let mut trap_last_send = Instant::now();

        while running.load(Ordering::Relaxed) {
            // Process commands (non-blocking)
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    TxCommand::StartForward(config) => {
                        forward_config = Some(config);
                    }
                    TxCommand::StopForward => {
                        forward_config = None;
                    }
                    TxCommand::StartTrapezoidal(config) => {
                        trap_current_value = config.initial_value;
                        trap_phase = TrapezoidPhase::Rising;
                        trap_phase_start = Instant::now();
                        trap_last_send = Instant::now();
                        trap_config = Some(config);
                    }
                    TxCommand::StopTrapezoidal => {
                        trap_config = None;
                    }
                    TxCommand::SendOnce {
                        channel_mask,
                        frame,
                    } => {
                        let port_guard = port.lock();
                        let _ = port_guard.transmit(channel_mask, &frame);
                    }
                    TxCommand::Shutdown => {
                        return;
                    }
                }
            }

            // Process forwarding
            if let Some(ref fwd) = forward_config {
                while let Ok(frame) = rx_frames.try_recv() {
                    if frame.direction != Direction::Rx {
                        continue;
                    }
                    if frame.channel != fwd.rx_channel {
                        continue;
                    }
                    if !fwd.message_filter.is_empty()
                        && !fwd.message_filter.contains(&frame.id)
                    {
                        continue;
                    }

                    let mut tx_frame = frame.clone();
                    tx_frame.direction = Direction::Tx;

                    // Apply signal modifications
                    if !fwd.modifications.is_empty() {
                        if let Some(ref dbc_db) = *dbc.read() {
                            for modification in &fwd.modifications {
                                if modification.message_id == tx_frame.id {
                                    if let Some(msg) =
                                        dbc_db.get_message(modification.message_id)
                                    {
                                        if let Some(signal) =
                                            msg.get_signal(&modification.signal_name)
                                        {
                                            let clamped = modification
                                                .value
                                                .clamp(signal.min, signal.max);
                                            signal.encode_physical(
                                                clamped,
                                                &mut tx_frame.data,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let port_guard = port.lock();
                    let _ = port_guard.transmit(fwd.tx_channel_mask, &tx_frame);
                }
            }

            // Process trapezoidal wave
            if let Some(ref config) = trap_config {
                let now = Instant::now();
                let cycle_duration = Duration::from_millis(config.cycle_time_ms);

                if now.duration_since(trap_last_send) >= cycle_duration {
                    let dt = now.duration_since(trap_phase_start).as_secs_f64();
                    let delta = config.rate * dt;

                    match trap_phase {
                        TrapezoidPhase::Rising => {
                            trap_current_value = config.initial_value + delta;
                            if trap_current_value >= config.max_value {
                                trap_current_value = config.max_value;
                                trap_phase = TrapezoidPhase::Hold;
                                trap_phase_start = now;
                            }
                        }
                        TrapezoidPhase::Hold => {
                            trap_current_value = config.max_value;
                            let hold_duration =
                                Duration::from_millis(config.hold_time_ms);
                            if now.duration_since(trap_phase_start) >= hold_duration {
                                trap_phase = TrapezoidPhase::Falling;
                                trap_phase_start = now;
                            }
                        }
                        TrapezoidPhase::Falling => {
                            trap_current_value = config.max_value - delta;
                            if trap_current_value <= config.initial_value {
                                trap_current_value = config.initial_value;
                                if config.repeat {
                                    trap_phase = TrapezoidPhase::Rising;
                                    trap_phase_start = now;
                                } else {
                                    trap_phase = TrapezoidPhase::Done;
                                }
                            }
                        }
                        TrapezoidPhase::Done => {
                            // One-shot completed, do nothing
                        }
                    }

                    if trap_phase != TrapezoidPhase::Done {
                        // Build and send the frame
                        let mut data = config.base_data.clone();

                        // Encode the signal value using DBC
                        if let Some(ref dbc_db) = *dbc.read() {
                            if let Some(msg) = dbc_db.get_message(config.message_id) {
                                if let Some(signal) = msg.get_signal(&config.signal_name)
                                {
                                    let clamped = trap_current_value
                                        .clamp(signal.min, signal.max);
                                    signal.encode_physical(clamped, &mut data);
                                }
                            }
                        }

                        let frame = if config.is_fd {
                            CanFrame::new_canfd(
                                0.0,
                                0,
                                config.message_id,
                                config.message_id > 0x7FF,
                                true,
                                config.dlc,
                                &data,
                                Direction::Tx,
                            )
                        } else {
                            CanFrame::new_can(
                                0.0,
                                0,
                                config.message_id,
                                config.message_id > 0x7FF,
                                config.dlc,
                                &data,
                                Direction::Tx,
                            )
                        };

                        let port_guard = port.lock();
                        let _ = port_guard.transmit(config.tx_channel_mask, &frame);
                        trap_last_send = now;
                    }
                }
            }

            // Small sleep to avoid busy-waiting
            thread::sleep(Duration::from_micros(100));
        }
    }

    pub fn stop(&mut self) {
        let _ = self.cmd_tx.send(TxCommand::Shutdown);
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CanTransmitter {
    fn drop(&mut self) {
        self.stop();
    }
}
