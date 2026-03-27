/// DBC Message definition
use serde::{Deserialize, Serialize};

use super::signal::DbcSignal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbcMessage {
    /// Message ID (11-bit or 29-bit)
    pub id: u32,
    /// Message name
    pub name: String,
    /// Data length in bytes
    pub dlc: u8,
    /// Transmitting node
    pub transmitter: String,
    /// Signals in this message
    pub signals: Vec<DbcSignal>,
}

impl DbcMessage {
    pub fn get_signal(&self, name: &str) -> Option<&DbcSignal> {
        self.signals.iter().find(|s| s.name == name)
    }

    pub fn get_signal_mut(&mut self, name: &str) -> Option<&mut DbcSignal> {
        self.signals.iter_mut().find(|s| s.name == name)
    }

    /// Decode all signals from CAN data and return (signal_name, physical_value) pairs
    pub fn decode_all(&self, data: &[u8]) -> Vec<(String, f64)> {
        self.signals
            .iter()
            .map(|s| (s.name.clone(), s.decode_physical(data)))
            .collect()
    }
}
