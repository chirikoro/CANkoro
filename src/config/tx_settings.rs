/// Transmission settings save/load
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::can::transmitter::{ForwardConfig, SignalModification, TrapezoidalConfig};
use crate::vector::types::XLaccess;

/// A single panel widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelWidget {
    Switch {
        id: String,
        label: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        /// Linked signal modification
        on_modification: Option<SignalModification>,
        off_modification: Option<SignalModification>,
        state: bool,
    },
    ValueBox {
        id: String,
        label: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        /// Linked signal
        message_id: Option<u32>,
        signal_name: Option<String>,
        value: f64,
        min: f64,
        max: f64,
        unit: String,
    },
    SelectBox {
        id: String,
        label: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        options: Vec<(String, f64)>,
        selected_index: usize,
        message_id: Option<u32>,
        signal_name: Option<String>,
    },
    Label {
        id: String,
        text: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        font_size: f32,
    },
}

impl PanelWidget {
    pub fn id(&self) -> &str {
        match self {
            PanelWidget::Switch { id, .. } => id,
            PanelWidget::ValueBox { id, .. } => id,
            PanelWidget::SelectBox { id, .. } => id,
            PanelWidget::Label { id, .. } => id,
        }
    }

    pub fn position(&self) -> (f32, f32) {
        match self {
            PanelWidget::Switch { x, y, .. } => (*x, *y),
            PanelWidget::ValueBox { x, y, .. } => (*x, *y),
            PanelWidget::SelectBox { x, y, .. } => (*x, *y),
            PanelWidget::Label { x, y, .. } => (*x, *y),
        }
    }

    pub fn set_position(&mut self, new_x: f32, new_y: f32) {
        match self {
            PanelWidget::Switch { x, y, .. } => {
                *x = new_x;
                *y = new_y;
            }
            PanelWidget::ValueBox { x, y, .. } => {
                *x = new_x;
                *y = new_y;
            }
            PanelWidget::SelectBox { x, y, .. } => {
                *x = new_x;
                *y = new_y;
            }
            PanelWidget::Label { x, y, .. } => {
                *x = new_x;
                *y = new_y;
            }
        }
    }
}

/// A user-defined transmission panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxPanel {
    pub name: String,
    pub widgets: Vec<PanelWidget>,
}

/// Transmission settings that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSettings {
    /// Forward configurations
    pub forwards: Vec<ForwardSetting>,
    /// Trapezoidal wave configurations
    pub trapezoidals: Vec<TrapezoidalSetting>,
    /// User-defined panels
    pub panels: Vec<TxPanel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardSetting {
    pub name: String,
    pub enabled: bool,
    pub rx_channel_index: u32,
    pub tx_channel_index: u32,
    pub message_filter: Vec<u32>,
    pub modifications: Vec<SignalModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapezoidalSetting {
    pub name: String,
    pub enabled: bool,
    pub tx_channel_index: u32,
    pub message_id: u32,
    pub signal_name: String,
    pub initial_value: f64,
    pub max_value: f64,
    pub rate: f64,
    pub hold_time_ms: u64,
    pub cycle_time_ms: u64,
    pub repeat: bool,
    pub base_data: Vec<u8>,
    pub is_fd: bool,
    pub dlc: u8,
}

impl TxSettings {
    pub fn new() -> Self {
        Self {
            forwards: Vec::new(),
            trapezoidals: Vec::new(),
            panels: Vec::new(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("Serialize error: {}", e))?;
        std::fs::write(path, json).map_err(|e| format!("Write error: {}", e))
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))
    }
}

impl Default for TxSettings {
    fn default() -> Self {
        Self::new()
    }
}
