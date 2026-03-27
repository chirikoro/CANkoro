/// Log file reader for playback (ASC and BLF)
use std::path::Path;

use crate::can::frame::CanFrame;
use crate::log::asc;

/// Supported log file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Asc,
    Blf,
}

impl LogFormat {
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_lowercase().as_str() {
            "asc" => Some(Self::Asc),
            "blf" => Some(Self::Blf),
            _ => None,
        }
    }
}

/// Load a log file and return all frames
pub fn load_log_file(path: &Path) -> Result<Vec<CanFrame>, String> {
    let format = LogFormat::from_extension(path)
        .ok_or_else(|| "Unsupported log file format".to_string())?;

    match format {
        LogFormat::Asc => asc::read_asc(path),
        LogFormat::Blf => read_blf(path),
    }
}

/// Read a BLF file (simplified reader)
fn read_blf(path: &Path) -> Result<Vec<CanFrame>, String> {
    use byteorder::{LittleEndian, ReadBytesExt};
    use flate2::read::ZlibDecoder;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    let data = std::fs::read(path).map_err(|e| format!("Failed to read BLF file: {}", e))?;
    let mut cursor = Cursor::new(&data);
    let mut frames = Vec::new();

    // Skip file header
    if data.len() < 144 {
        return Err("BLF file too small".to_string());
    }
    cursor.seek(SeekFrom::Start(144)).map_err(|e| e.to_string())?;

    while (cursor.position() as usize) < data.len() - 16 {
        // Read object header
        let signature = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        if signature != 0x4A424F4C {
            // Not LOBJ
            break;
        }

        let header_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        let _header_ver = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        let obj_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let obj_type = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // v1 header extension
        let timestamp = cursor.read_u64::<LittleEndian>().unwrap_or(0);
        let _reserved = cursor.read_u64::<LittleEndian>().unwrap_or(0);

        let data_start = cursor.position();
        let data_size = obj_size as u64 - header_size as u64 - 16; // subtract v1 extension

        match obj_type {
            1 => {
                // CAN message
                if data_size >= 16 {
                    let channel = cursor.read_u16::<LittleEndian>().unwrap_or(0) as u8;
                    let dlc = cursor.read_u8().unwrap_or(0);
                    let flags = cursor.read_u8().unwrap_or(0);
                    let id_raw = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                    let is_extended = id_raw & 0x80000000 != 0;
                    let id = id_raw & 0x1FFFFFFF;
                    let mut can_data = [0u8; 8];
                    let _ = cursor.read_exact(&mut can_data);
                    let direction = if flags & 1 != 0 {
                        crate::can::frame::Direction::Tx
                    } else {
                        crate::can::frame::Direction::Rx
                    };
                    let ts = timestamp as f64 / 1_000_000_000.0;
                    frames.push(CanFrame::new_can(
                        ts,
                        channel,
                        id,
                        is_extended,
                        dlc,
                        &can_data[..dlc.min(8) as usize],
                        direction,
                    ));
                }
            }
            100 => {
                // CAN FD message
                if data_size >= 12 {
                    let channel = cursor.read_u16::<LittleEndian>().unwrap_or(0) as u8;
                    let dlc = cursor.read_u8().unwrap_or(0);
                    let flags = cursor.read_u8().unwrap_or(0);
                    let id_raw = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                    let is_extended = id_raw & 0x80000000 != 0;
                    let id = id_raw & 0x1FFFFFFF;
                    let fd_flags = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                    let is_fd = fd_flags & 0x01 != 0;
                    let is_brs = fd_flags & 0x02 != 0;
                    let len = crate::vector::types::dlc_to_len(dlc);
                    let mut can_data = vec![0u8; len];
                    let _ = cursor.read_exact(&mut can_data);
                    let direction = if flags & 1 != 0 {
                        crate::can::frame::Direction::Tx
                    } else {
                        crate::can::frame::Direction::Rx
                    };
                    let ts = timestamp as f64 / 1_000_000_000.0;
                    if is_fd {
                        frames.push(CanFrame::new_canfd(
                            ts, channel, id, is_extended, is_brs, dlc, &can_data, direction,
                        ));
                    } else {
                        frames.push(CanFrame::new_can(
                            ts, channel, id, is_extended, dlc, &can_data, direction,
                        ));
                    }
                }
            }
            10 => {
                // Container - decompress and parse contained objects
                if data_size >= 8 {
                    let compressed_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                    let _uncompressed_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                    let mut compressed = vec![0u8; compressed_size as usize];
                    let _ = cursor.read_exact(&mut compressed);
                    if let Ok(decoder) = {
                        let result: Result<Vec<u8>, _> = {
                            let mut dec = ZlibDecoder::new(&compressed[..]);
                            let mut buf = Vec::new();
                            dec.read_to_end(&mut buf).map(|_| buf)
                        };
                        result
                    } {
                        // Parse objects from decompressed data (recursive would be complex,
                        // skip nested containers for simplicity)
                        let nested_data = decoder;
                        let mut nc = Cursor::new(&nested_data);
                        while (nc.position() as usize) < nested_data.len().saturating_sub(16) {
                            let sig = nc.read_u32::<LittleEndian>().unwrap_or(0);
                            if sig != 0x4A424F4C {
                                break;
                            }
                            let _hs = nc.read_u16::<LittleEndian>().unwrap_or(0);
                            let _hv = nc.read_u16::<LittleEndian>().unwrap_or(0);
                            let os = nc.read_u32::<LittleEndian>().unwrap_or(0);
                            let ot = nc.read_u32::<LittleEndian>().unwrap_or(0);
                            let ts2 = nc.read_u64::<LittleEndian>().unwrap_or(0);
                            let _r = nc.read_u64::<LittleEndian>().unwrap_or(0);

                            match ot {
                                1 => {
                                    let ch = nc.read_u16::<LittleEndian>().unwrap_or(0) as u8;
                                    let dlc = nc.read_u8().unwrap_or(0);
                                    let fl = nc.read_u8().unwrap_or(0);
                                    let ir = nc.read_u32::<LittleEndian>().unwrap_or(0);
                                    let ie = ir & 0x80000000 != 0;
                                    let iid = ir & 0x1FFFFFFF;
                                    let mut cd = [0u8; 8];
                                    let _ = nc.read_exact(&mut cd);
                                    let dir = if fl & 1 != 0 {
                                        crate::can::frame::Direction::Tx
                                    } else {
                                        crate::can::frame::Direction::Rx
                                    };
                                    frames.push(CanFrame::new_can(
                                        ts2 as f64 / 1e9,
                                        ch, iid, ie, dlc,
                                        &cd[..dlc.min(8) as usize],
                                        dir,
                                    ));
                                }
                                _ => {
                                    // Skip unknown object
                                    let skip = os as i64 - 32;
                                    if skip > 0 {
                                        let _ = nc.seek(SeekFrom::Current(skip));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // Seek to next object (align to 4 bytes)
        let next_pos = data_start + (obj_size as u64 - 32); // subtract header already read
        let aligned = (next_pos + 3) & !3;
        if aligned > cursor.position() {
            cursor
                .seek(SeekFrom::Start(aligned))
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(frames)
}
