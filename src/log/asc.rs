/// Vector ASC (ASCII) log file format writer and reader
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use chrono::Local;

use crate::can::frame::{CanFrame, Direction};

/// ASC file writer (Vector format)
pub struct AscWriter {
    writer: BufWriter<std::fs::File>,
    start_time: f64,
}

impl AscWriter {
    pub fn new(path: &Path) -> Result<Self, String> {
        let file =
            std::fs::File::create(path).map_err(|e| format!("Failed to create ASC file: {}", e))?;
        let mut writer = BufWriter::with_capacity(65536, file);

        // Write header
        let now = Local::now();
        let date_str = now.format("%a %b %d %I:%M:%S %p %Y").to_string();
        writeln!(writer, "date {}", date_str).map_err(|e| e.to_string())?;
        writeln!(writer, "base hex  timestamps absolute").map_err(|e| e.to_string())?;
        writeln!(writer, "internal events logged").map_err(|e| e.to_string())?;
        writeln!(writer, "// CANkoro ASC Log").map_err(|e| e.to_string())?;

        Ok(Self {
            writer,
            start_time: 0.0,
        })
    }

    pub fn set_start_time(&mut self, t: f64) {
        self.start_time = t;
    }

    /// Write a CAN frame to the ASC file
    pub fn write_frame(&mut self, frame: &CanFrame) -> Result<(), String> {
        let timestamp = frame.timestamp - self.start_time;
        let dir = match frame.direction {
            Direction::Rx => "Rx",
            Direction::Tx => "Tx",
        };
        let channel = frame.channel + 1; // ASC channels are 1-based

        if frame.is_fd {
            // CAN FD format
            let data_str = frame
                .data
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let flags = if frame.is_brs { "1" } else { "0" };
            let esi = if frame.is_esi { "1" } else { "0" };
            writeln!(
                self.writer,
                "  {:.6} CANFD   {:03x}             {} d {} {} {} {} {}    0    0    0    0    0",
                timestamp,
                frame.id,
                dir,
                frame.dlc,
                frame.data.len(),
                flags,
                esi,
                data_str
            )
            .map_err(|e| e.to_string())?;
        } else {
            // Classic CAN format:
            // timestamp channel id dir dlc data
            let data_str = frame
                .data
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");

            if frame.is_extended {
                writeln!(
                    self.writer,
                    "  {:.6} {}  {:08X}x       {} d {} {}",
                    timestamp, channel, frame.id, dir, frame.dlc, data_str
                )
                .map_err(|e| e.to_string())?;
            } else {
                writeln!(
                    self.writer,
                    "  {:.6} {}  {:03X}             {} d {} {}",
                    timestamp, channel, frame.id, dir, frame.dlc, data_str
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), String> {
        self.writer.flush().map_err(|e| e.to_string())
    }
}

impl Drop for AscWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Parse an ASC file and return frames
pub fn read_asc(path: &Path) -> Result<Vec<CanFrame>, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open ASC file: {}", e))?;
    let reader = BufReader::new(file);
    let mut frames = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let trimmed = line.trim();

        // Skip header lines
        if trimmed.is_empty()
            || trimmed.starts_with("date")
            || trimmed.starts_with("base")
            || trimmed.starts_with("internal")
            || trimmed.starts_with("//")
            || trimmed.starts_with("Begin")
            || trimmed.starts_with("End")
        {
            continue;
        }

        if let Some(frame) = parse_asc_line(trimmed) {
            frames.push(frame);
        }
    }

    Ok(frames)
}

fn parse_asc_line(line: &str) -> Option<CanFrame> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    // Parse timestamp
    let timestamp: f64 = parts[0].parse().ok()?;

    // Check for CANFD
    if parts[1] == "CANFD" {
        return parse_asc_canfd_line(&parts, timestamp);
    }

    // Parse channel (1-based in ASC)
    let channel: u8 = parts[1].parse::<u8>().ok()?.saturating_sub(1);

    // Parse ID
    let id_str = parts[2].trim_end_matches('x');
    let is_extended = parts[2].ends_with('x');
    let id = u32::from_str_radix(id_str, 16).ok()?;

    // Parse direction
    let direction = match parts[3] {
        "Rx" => Direction::Rx,
        "Tx" => Direction::Tx,
        _ => Direction::Rx,
    };

    // parts[4] should be 'd' for data frame
    if parts[4] != "d" {
        return None;
    }

    // Parse DLC
    let dlc: u8 = parts[5].parse().ok()?;

    // Parse data bytes
    let mut data = Vec::new();
    for i in 6..parts.len().min(6 + dlc as usize) {
        if let Ok(byte) = u8::from_str_radix(parts[i], 16) {
            data.push(byte);
        }
    }

    Some(CanFrame::new_can(
        timestamp,
        channel,
        id,
        is_extended,
        dlc,
        &data,
        direction,
    ))
}

fn parse_asc_canfd_line(parts: &[&str], timestamp: f64) -> Option<CanFrame> {
    if parts.len() < 8 {
        return None;
    }

    // CANFD format: timestamp CANFD id dir d dlc len brs esi data...
    let id = u32::from_str_radix(parts[2], 16).ok()?;
    let direction = match parts[3] {
        "Rx" => Direction::Rx,
        "Tx" => Direction::Tx,
        _ => Direction::Rx,
    };

    // parts[4] = 'd'
    let dlc: u8 = parts[5].parse().ok()?;
    let _len: usize = parts[6].parse().ok()?;
    let is_brs = parts[7] == "1";
    // parts[8] = esi

    let data_start = 9;
    let mut data = Vec::new();
    for i in data_start..parts.len() {
        if let Ok(byte) = u8::from_str_radix(parts[i], 16) {
            data.push(byte);
        }
    }

    Some(CanFrame::new_canfd(
        timestamp,
        0, // CANFD lines don't always have channel in this format
        id,
        id > 0x7FF,
        is_brs,
        dlc,
        &data,
        direction,
    ))
}
