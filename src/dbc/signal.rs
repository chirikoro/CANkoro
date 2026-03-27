/// DBC Signal definition and physical value conversion
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbcSignal {
    pub name: String,
    pub start_bit: u32,
    pub bit_length: u32,
    pub byte_order: ByteOrder,
    pub is_signed: bool,
    pub factor: f64,
    pub offset: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    pub receivers: Vec<String>,
    pub value_descriptions: Option<HashMap<i64, String>>,
}

impl DbcSignal {
    /// Extract raw value from CAN data bytes
    pub fn extract_raw(&self, data: &[u8]) -> u64 {
        match self.byte_order {
            ByteOrder::LittleEndian => self.extract_raw_le(data),
            ByteOrder::BigEndian => self.extract_raw_be(data),
        }
    }

    fn extract_raw_le(&self, data: &[u8]) -> u64 {
        let mut value: u64 = 0;
        for i in 0..self.bit_length {
            let bit_pos = self.start_bit + i;
            let byte_idx = (bit_pos / 8) as usize;
            let bit_idx = bit_pos % 8;
            if byte_idx < data.len() {
                if data[byte_idx] & (1 << bit_idx) != 0 {
                    value |= 1 << i;
                }
            }
        }
        value
    }

    fn extract_raw_be(&self, data: &[u8]) -> u64 {
        let mut value: u64 = 0;
        let start_byte = (self.start_bit / 8) as usize;
        let start_bit_in_byte = (self.start_bit % 8) as usize;

        let mut bit_count = 0u32;
        let mut byte_idx = start_byte;
        let mut bit_idx = start_bit_in_byte as i32;

        while bit_count < self.bit_length {
            if byte_idx < data.len() {
                if data[byte_idx] & (1 << bit_idx) != 0 {
                    value |= 1 << (self.bit_length - 1 - bit_count);
                }
            }
            bit_count += 1;
            bit_idx -= 1;
            if bit_idx < 0 {
                bit_idx = 7;
                byte_idx += 1;
            }
        }
        value
    }

    /// Convert raw value to physical value
    pub fn raw_to_physical(&self, raw: u64) -> f64 {
        let raw_float = if self.is_signed {
            // Sign extend
            let mask = 1u64 << (self.bit_length - 1);
            let val = raw & ((1u64 << self.bit_length) - 1);
            if val & mask != 0 {
                // Negative
                let extended = val | !((1u64 << self.bit_length) - 1);
                extended as i64 as f64
            } else {
                val as f64
            }
        } else {
            raw as f64
        };
        raw_float * self.factor + self.offset
    }

    /// Convert physical value to raw value
    pub fn physical_to_raw(&self, physical: f64) -> u64 {
        let raw_float = (physical - self.offset) / self.factor;
        if self.is_signed {
            let raw_int = raw_float.round() as i64;
            let mask = (1u64 << self.bit_length) - 1;
            (raw_int as u64) & mask
        } else {
            let raw_uint = raw_float.round().max(0.0) as u64;
            let mask = (1u64 << self.bit_length) - 1;
            raw_uint & mask
        }
    }

    /// Decode physical value from CAN data
    pub fn decode_physical(&self, data: &[u8]) -> f64 {
        let raw = self.extract_raw(data);
        self.raw_to_physical(raw)
    }

    /// Encode physical value into CAN data bytes
    pub fn encode_physical(&self, physical: f64, data: &mut [u8]) {
        let clamped = physical.clamp(self.min, self.max);
        let raw = self.physical_to_raw(clamped);
        self.encode_raw(raw, data);
    }

    /// Encode raw value into CAN data bytes
    pub fn encode_raw(&self, raw: u64, data: &mut [u8]) {
        match self.byte_order {
            ByteOrder::LittleEndian => self.encode_raw_le(raw, data),
            ByteOrder::BigEndian => self.encode_raw_be(raw, data),
        }
    }

    fn encode_raw_le(&self, raw: u64, data: &mut [u8]) {
        for i in 0..self.bit_length {
            let bit_pos = self.start_bit + i;
            let byte_idx = (bit_pos / 8) as usize;
            let bit_idx = bit_pos % 8;
            if byte_idx < data.len() {
                if raw & (1 << i) != 0 {
                    data[byte_idx] |= 1 << bit_idx;
                } else {
                    data[byte_idx] &= !(1 << bit_idx);
                }
            }
        }
    }

    fn encode_raw_be(&self, raw: u64, data: &mut [u8]) {
        let start_byte = (self.start_bit / 8) as usize;
        let start_bit_in_byte = (self.start_bit % 8) as usize;

        let mut bit_count = 0u32;
        let mut byte_idx = start_byte;
        let mut bit_idx = start_bit_in_byte as i32;

        while bit_count < self.bit_length {
            if byte_idx < data.len() {
                if raw & (1 << (self.bit_length - 1 - bit_count)) != 0 {
                    data[byte_idx] |= 1 << bit_idx;
                } else {
                    data[byte_idx] &= !(1 << bit_idx as u32);
                }
            }
            bit_count += 1;
            bit_idx -= 1;
            if bit_idx < 0 {
                bit_idx = 7;
                byte_idx += 1;
            }
        }
    }

    /// Get the calculated min raw value
    pub fn min_raw(&self) -> u64 {
        self.physical_to_raw(self.min)
    }

    /// Get the calculated max raw value
    pub fn max_raw(&self) -> u64 {
        self.physical_to_raw(self.max)
    }
}
