/// High-level wrapper around Vector XL Driver Library
use std::sync::Arc;

use super::types::*;
use super::xlapi::XlApi;

/// Information about a detected hardware channel
#[derive(Debug, Clone)]
pub struct HwChannelInfo {
    pub channel_index: u32,
    pub channel_mask: XLaccess,
    pub hw_type: u32,
    pub hw_index: u32,
    pub hw_channel: u32,
    pub name: String,
    pub serial_number: u32,
    pub transceiver_name: String,
    pub is_on_bus: bool,
}

impl HwChannelInfo {
    /// Returns true if this is a Virtual CAN channel
    pub fn is_virtual(&self) -> bool {
        self.hw_type == XL_HWTYPE_VIRTUAL
    }

    /// Human-readable hardware type name
    pub fn hw_type_name(&self) -> &'static str {
        match self.hw_type {
            XL_HWTYPE_VIRTUAL => "Virtual",
            XL_HWTYPE_CANCARDXL => "CANcardXL",
            XL_HWTYPE_VN1610 => "VN1610",
            XL_HWTYPE_VN1630 => "VN1630",
            XL_HWTYPE_VN1640 => "VN1640",
            XL_HWTYPE_VN1670 => "VN1670",
            _ => "Unknown",
        }
    }
}

/// Vector driver manager
pub struct VectorDriver {
    api: Arc<XlApi>,
    is_open: bool,
}

impl VectorDriver {
    /// Open the driver and load the XL API
    pub fn new() -> Result<Self, String> {
        let api = XlApi::load()?;
        let status = unsafe { (api.xl_open_driver)() };
        if status != XL_SUCCESS {
            return Err(format!(
                "xlOpenDriver failed: {} ({})",
                xl_status_to_string(status),
                status
            ));
        }

        // Log struct sizes for debugging
        log::info!(
            "XLchannelConfig size: {} bytes (expected 224)",
            std::mem::size_of::<XLchannelConfig>()
        );
        log::info!(
            "XLdriverConfig size: {} bytes (expected 14384)",
            std::mem::size_of::<XLdriverConfig>()
        );

        Ok(Self {
            api: Arc::new(api),
            is_open: true,
        })
    }

    pub fn api(&self) -> &Arc<XlApi> {
        &self.api
    }

    /// Get all available hardware channels
    pub fn get_channels(&self) -> Result<Vec<HwChannelInfo>, String> {
        let mut config = XLdriverConfig::default();
        let status = unsafe { (self.api.xl_get_driver_config)(&mut config) };
        if status != XL_SUCCESS {
            return Err(format!(
                "xlGetDriverConfig failed: {} ({})",
                xl_status_to_string(status),
                status
            ));
        }

        let channel_count = config.channelCount;
        log::info!("xlGetDriverConfig: {} channels found", channel_count);

        let mut channels = Vec::new();
        for i in 0..channel_count as usize {
            if i >= XL_CONFIG_MAX_CHANNELS {
                break;
            }
            let ch = &config.channel[i];

            // Read packed fields into local variables (avoid unaligned access issues)
            let hw_type = ch.hwType as u32;
            let hw_index = ch.hwIndex as u32;
            let hw_channel = ch.hwChannel as u32;
            let channel_index = ch.channelIndex as u32;
            let channel_mask = ch.channelMask;
            let bus_capabilities = ch.channelBusCapabilities;
            let connected_bus = ch.connectedBusType;
            let is_on_bus = ch.isOnBus;
            let serial = ch.serialNumber;

            let name = String::from_utf8_lossy(
                &ch.name[..ch.name.iter().position(|&b| b == 0).unwrap_or(ch.name.len())],
            )
            .to_string();

            let transceiver_name = String::from_utf8_lossy(
                &ch.transceiverName[..ch
                    .transceiverName
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(ch.transceiverName.len())],
            )
            .to_string();

            log::info!(
                "  Channel {}: name='{}' hwType={} hwIndex={} hwChannel={} mask=0x{:X} busCap=0x{:X} connBus=0x{:X} serial={}",
                i, name, hw_type, hw_index, hw_channel, channel_mask, bus_capabilities, connected_bus, serial
            );

            // Filter for CAN-capable channels:
            // Virtual channels: always include (hw_type == XL_HWTYPE_VIRTUAL)
            // Physical hardware: check channelBusCapabilities or connectedBusType
            let is_virtual = hw_type == XL_HWTYPE_VIRTUAL;
            let is_can_capable = bus_capabilities & XL_BUS_TYPE_CAN != 0
                || connected_bus == XL_BUS_TYPE_CAN;
            if !is_virtual && !is_can_capable {
                log::info!("    -> Skipped (not CAN capable and not Virtual)");
                continue;
            }

            channels.push(HwChannelInfo {
                channel_index,
                channel_mask,
                hw_type,
                hw_index,
                hw_channel,
                name,
                serial_number: serial,
                transceiver_name,
                is_on_bus: is_on_bus != 0,
            });
        }

        log::info!("{} CAN-capable channels after filtering", channels.len());
        Ok(channels)
    }
}

impl Drop for VectorDriver {
    fn drop(&mut self) {
        if self.is_open {
            unsafe {
                (self.api.xl_close_driver)();
            }
            self.is_open = false;
        }
    }
}
