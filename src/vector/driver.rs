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

        let mut channels = Vec::new();
        for i in 0..config.channelCount as usize {
            if i >= XL_CONFIG_MAX_CHANNELS {
                break;
            }
            let ch = &config.channel[i];
            // Filter for CAN-capable channels:
            // Physical hardware: check channelBusCapabilities or connectedBusType
            // Virtual channels: always include (hw_type == XL_HWTYPE_VIRTUAL)
            let is_virtual = ch.hw_type == XL_HWTYPE_VIRTUAL;
            let is_can_capable = ch.channelBusCapabilities & XL_BUS_TYPE_CAN != 0
                || ch.connectedBusType == XL_BUS_TYPE_CAN;
            if !is_virtual && !is_can_capable {
                continue;
            }

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

            channels.push(HwChannelInfo {
                channel_index: ch.channelIndex,
                channel_mask: ch.channelMask,
                hw_type: ch.hw_type,
                hw_index: ch.hw_index,
                hw_channel: ch.hw_channel,
                name,
                serial_number: ch.serialNumber,
                transceiver_name,
                is_on_bus: ch.isOnBus != 0,
            });
        }
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
