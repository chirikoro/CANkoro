/// Vector BLF (Binary Logging Format) writer
/// BLF format: File header + compressed object containers
use std::io::{BufWriter, Write};
use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::can::frame::{CanFrame, Direction};

// BLF signatures and constants
const BLF_FILE_SIGNATURE: &[u8; 7] = b"BL\x4E\x4B\x01\x00\x00";
const BLF_OBJECT_SIGNATURE: u32 = 0x4A424F4C; // "LOBJ"
const BLF_OBJTYPE_CAN_MESSAGE: u32 = 1;
const BLF_OBJTYPE_CAN_FD_MESSAGE: u32 = 100;
const BLF_OBJTYPE_CONTAINER: u32 = 10;
const BLF_HEADER_SIZE: u32 = 144;
const BLF_OBJ_HEADER_BASE_SIZE: u32 = 16;
const BLF_OBJ_HEADER_V1_SIZE: u32 = 32;

/// BLF file writer
pub struct BlfWriter {
    writer: BufWriter<std::fs::File>,
    object_count: u32,
    start_timestamp: u64,
    last_timestamp: u64,
    uncompressed_size: u64,
    buffer: Vec<u8>,
    buffer_threshold: usize,
}

impl BlfWriter {
    pub fn new(path: &Path) -> Result<Self, String> {
        let file =
            std::fs::File::create(path).map_err(|e| format!("Failed to create BLF file: {}", e))?;
        let mut writer = BufWriter::with_capacity(131072, file);

        // Write file header placeholder (will be updated on close)
        let header = [0u8; BLF_HEADER_SIZE as usize];
        writer.write_all(&header).map_err(|e| e.to_string())?;

        Ok(Self {
            writer,
            object_count: 0,
            start_timestamp: 0,
            last_timestamp: 0,
            uncompressed_size: 0,
            buffer: Vec::with_capacity(65536),
            buffer_threshold: 65536,
        })
    }

    /// Write a CAN frame
    pub fn write_frame(&mut self, frame: &CanFrame) -> Result<(), String> {
        let timestamp_ns = (frame.timestamp * 1_000_000_000.0) as u64;

        if self.object_count == 0 {
            self.start_timestamp = timestamp_ns;
        }
        self.last_timestamp = timestamp_ns;

        if frame.is_fd {
            self.write_canfd_object(frame, timestamp_ns)?;
        } else {
            self.write_can_object(frame, timestamp_ns)?;
        }

        self.object_count += 1;

        if self.buffer.len() >= self.buffer_threshold {
            self.flush_buffer()?;
        }

        Ok(())
    }

    fn write_can_object(&mut self, frame: &CanFrame, timestamp_ns: u64) -> Result<(), String> {
        let data_len = frame.data.len().min(8);
        // CAN message object: channel(2) + dlc(1) + flags(1) + id(4) + data(8) = 16
        let obj_data_size = 16u32;
        let total_size = BLF_OBJ_HEADER_V1_SIZE + obj_data_size;

        // Object header base (LOBJ)
        self.buffer
            .write_u32::<LittleEndian>(BLF_OBJECT_SIGNATURE)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u16::<LittleEndian>(BLF_OBJ_HEADER_BASE_SIZE as u16)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u16::<LittleEndian>(1) // header version
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u32::<LittleEndian>(total_size)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u32::<LittleEndian>(BLF_OBJTYPE_CAN_MESSAGE)
            .map_err(|e| e.to_string())?;

        // Object header v1 extension (timestamp)
        self.buffer
            .write_u64::<LittleEndian>(timestamp_ns)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u64::<LittleEndian>(0) // reserved
            .map_err(|e| e.to_string())?;

        // CAN message data
        self.buffer
            .write_u16::<LittleEndian>(frame.channel as u16)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u8(frame.dlc)
            .map_err(|e| e.to_string())?;

        let flags: u8 = match frame.direction {
            Direction::Tx => 1,
            Direction::Rx => 0,
        };
        self.buffer.write_u8(flags).map_err(|e| e.to_string())?;

        let mut id = frame.id;
        if frame.is_extended {
            id |= 0x80000000;
        }
        self.buffer
            .write_u32::<LittleEndian>(id)
            .map_err(|e| e.to_string())?;

        // Write data (padded to 8 bytes)
        let mut data_buf = [0u8; 8];
        data_buf[..data_len].copy_from_slice(&frame.data[..data_len]);
        self.buffer
            .write_all(&data_buf)
            .map_err(|e| e.to_string())?;

        self.uncompressed_size += total_size as u64;
        Ok(())
    }

    fn write_canfd_object(
        &mut self,
        frame: &CanFrame,
        timestamp_ns: u64,
    ) -> Result<(), String> {
        let data_len = frame.data.len().min(64);
        // CAN FD object: channel(2) + dlc(1) + flags(1) + id(4) + fdflags(4) + data(variable)
        let obj_data_size = 12u32 + data_len as u32;
        let total_size = BLF_OBJ_HEADER_V1_SIZE + obj_data_size;
        // Align to 4 bytes
        let padded_total = (total_size + 3) & !3;

        // Object header base
        self.buffer
            .write_u32::<LittleEndian>(BLF_OBJECT_SIGNATURE)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u16::<LittleEndian>(BLF_OBJ_HEADER_BASE_SIZE as u16)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u16::<LittleEndian>(1)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u32::<LittleEndian>(padded_total)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u32::<LittleEndian>(BLF_OBJTYPE_CAN_FD_MESSAGE)
            .map_err(|e| e.to_string())?;

        // Object header v1 extension
        self.buffer
            .write_u64::<LittleEndian>(timestamp_ns)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u64::<LittleEndian>(0)
            .map_err(|e| e.to_string())?;

        // CAN FD message data
        self.buffer
            .write_u16::<LittleEndian>(frame.channel as u16)
            .map_err(|e| e.to_string())?;
        self.buffer
            .write_u8(frame.dlc)
            .map_err(|e| e.to_string())?;

        let flags: u8 = match frame.direction {
            Direction::Tx => 1,
            Direction::Rx => 0,
        };
        self.buffer.write_u8(flags).map_err(|e| e.to_string())?;

        let mut id = frame.id;
        if frame.is_extended {
            id |= 0x80000000;
        }
        self.buffer
            .write_u32::<LittleEndian>(id)
            .map_err(|e| e.to_string())?;

        // FD flags
        let mut fd_flags: u32 = 0;
        if frame.is_fd {
            fd_flags |= 0x01; // EDL
        }
        if frame.is_brs {
            fd_flags |= 0x02; // BRS
        }
        if frame.is_esi {
            fd_flags |= 0x04; // ESI
        }
        self.buffer
            .write_u32::<LittleEndian>(fd_flags)
            .map_err(|e| e.to_string())?;

        // Data
        self.buffer
            .write_all(&frame.data[..data_len])
            .map_err(|e| e.to_string())?;

        // Padding
        let padding = (padded_total - total_size) as usize;
        for _ in 0..padding {
            self.buffer.write_u8(0).map_err(|e| e.to_string())?;
        }

        self.uncompressed_size += padded_total as u64;
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), String> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Compress buffer
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&self.buffer)
            .map_err(|e| e.to_string())?;
        let compressed = encoder.finish().map_err(|e| e.to_string())?;

        let uncompressed_size = self.buffer.len() as u32;
        let compressed_size = compressed.len() as u32;

        // Write container object
        let container_header_size = 8u32; // compressed_size(4) + uncompressed_size(4)
        let total_size = BLF_OBJ_HEADER_V1_SIZE + container_header_size + compressed_size;
        let padded_total = (total_size + 3) & !3;

        // Object header
        self.writer
            .write_u32::<LittleEndian>(BLF_OBJECT_SIGNATURE)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_u16::<LittleEndian>(BLF_OBJ_HEADER_BASE_SIZE as u16)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_u16::<LittleEndian>(1)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_u32::<LittleEndian>(padded_total)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_u32::<LittleEndian>(BLF_OBJTYPE_CONTAINER)
            .map_err(|e| e.to_string())?;

        // v1 extension
        self.writer
            .write_u64::<LittleEndian>(0)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_u64::<LittleEndian>(0)
            .map_err(|e| e.to_string())?;

        // Container data
        self.writer
            .write_u32::<LittleEndian>(compressed_size)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_u32::<LittleEndian>(uncompressed_size)
            .map_err(|e| e.to_string())?;
        self.writer
            .write_all(&compressed)
            .map_err(|e| e.to_string())?;

        // Padding
        let padding = (padded_total - total_size) as usize;
        for _ in 0..padding {
            self.writer.write_u8(0).map_err(|e| e.to_string())?;
        }

        self.buffer.clear();
        Ok(())
    }

    /// Finalize and close the BLF file
    pub fn finalize(&mut self) -> Result<(), String> {
        self.flush_buffer()?;

        // Seek to beginning and write proper file header
        use std::io::Seek;
        let file_size = self.writer.stream_position().map_err(|e| e.to_string())?;

        self.writer
            .seek(std::io::SeekFrom::Start(0))
            .map_err(|e| e.to_string())?;

        // File signature
        self.writer
            .write_all(BLF_FILE_SIGNATURE)
            .map_err(|e| e.to_string())?;
        // Statistics size
        self.writer
            .write_u32::<LittleEndian>(BLF_HEADER_SIZE)
            .map_err(|e| e.to_string())?;
        // API version
        self.writer
            .write_u32::<LittleEndian>(0x0403)
            .map_err(|e| e.to_string())?;
        // Platform
        self.writer
            .write_u32::<LittleEndian>(1)
            .map_err(|e| e.to_string())?; // Windows
        // Creation flags
        self.writer
            .write_u32::<LittleEndian>(0)
            .map_err(|e| e.to_string())?;
        // Measurement start time
        self.writer
            .write_u64::<LittleEndian>(self.start_timestamp)
            .map_err(|e| e.to_string())?;
        // Last object timestamp
        self.writer
            .write_u64::<LittleEndian>(self.last_timestamp)
            .map_err(|e| e.to_string())?;
        // Object count
        self.writer
            .write_u32::<LittleEndian>(self.object_count)
            .map_err(|e| e.to_string())?;
        // Object read (0)
        self.writer
            .write_u32::<LittleEndian>(0)
            .map_err(|e| e.to_string())?;
        // File size
        self.writer
            .write_u64::<LittleEndian>(file_size)
            .map_err(|e| e.to_string())?;
        // Uncompressed file size
        self.writer
            .write_u64::<LittleEndian>(self.uncompressed_size + BLF_HEADER_SIZE as u64)
            .map_err(|e| e.to_string())?;
        // Object count (again for compat)
        self.writer
            .write_u32::<LittleEndian>(self.object_count)
            .map_err(|e| e.to_string())?;

        self.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Drop for BlfWriter {
    fn drop(&mut self) {
        let _ = self.finalize();
    }
}
