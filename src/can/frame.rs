use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Rx,
    Tx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanFrame {
    /// Timestamp in seconds (high precision)
    pub timestamp: f64,
    /// Channel index (0-based)
    pub channel: u8,
    /// CAN ID (11-bit or 29-bit)
    pub id: u32,
    /// CAN FD frame
    pub is_fd: bool,
    /// Extended ID (29-bit)
    pub is_extended: bool,
    /// Bit Rate Switch (CAN FD)
    pub is_brs: bool,
    /// Error State Indicator (CAN FD)
    pub is_esi: bool,
    /// Remote Transmission Request
    pub is_rtr: bool,
    /// Data Length Code
    pub dlc: u8,
    /// Data bytes (up to 64 for CAN-FD)
    pub data: Vec<u8>,
    /// Direction (Rx/Tx)
    pub direction: Direction,
}

impl CanFrame {
    pub fn new_can(
        timestamp: f64,
        channel: u8,
        id: u32,
        is_extended: bool,
        dlc: u8,
        data: &[u8],
        direction: Direction,
    ) -> Self {
        Self {
            timestamp,
            channel,
            id,
            is_fd: false,
            is_extended,
            is_brs: false,
            is_esi: false,
            is_rtr: false,
            dlc,
            data: data.to_vec(),
            direction,
        }
    }

    pub fn new_canfd(
        timestamp: f64,
        channel: u8,
        id: u32,
        is_extended: bool,
        is_brs: bool,
        dlc: u8,
        data: &[u8],
        direction: Direction,
    ) -> Self {
        Self {
            timestamp,
            channel,
            id,
            is_fd: true,
            is_extended,
            is_brs,
            is_esi: false,
            is_rtr: false,
            dlc,
            data: data.to_vec(),
            direction,
        }
    }

    /// Get the actual data length based on DLC
    pub fn data_length(&self) -> usize {
        if self.is_fd {
            crate::vector::types::dlc_to_len(self.dlc)
        } else {
            self.dlc.min(8) as usize
        }
    }

    /// Format CAN ID as hex string
    pub fn id_hex(&self) -> String {
        if self.is_extended {
            format!("{:08X}", self.id)
        } else {
            format!("{:03X}", self.id)
        }
    }

    /// Format data bytes as hex string
    pub fn data_hex(&self) -> String {
        self.data
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
