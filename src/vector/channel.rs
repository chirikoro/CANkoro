/// CAN/CAN-FD channel management - open, configure, send, receive
use std::sync::Arc;

use super::types::*;
use super::xlapi::XlApi;
use crate::can::frame::{CanFrame, Direction};

/// CAN channel mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanMode {
    /// Classic CAN
    Can,
    /// CAN FD
    CanFd,
}

/// Configuration for opening a CAN channel
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel_mask: XLaccess,
    pub mode: CanMode,
    pub bitrate: u32,
    /// CAN FD data bitrate (only used in CanFd mode)
    pub data_bitrate: u32,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            channel_mask: 0,
            mode: CanMode::Can,
            bitrate: 500000,
            data_bitrate: 2000000,
        }
    }
}

/// An opened CAN port with one or more channels
pub struct CanPort {
    api: Arc<XlApi>,
    port_handle: XLportHandle,
    permission_mask: XLaccess,
    access_mask: XLaccess,
    notification_handle: XLhandle,
    mode: CanMode,
    is_active: bool,
}

impl CanPort {
    /// Open a CAN port on the specified channels
    pub fn open(
        api: Arc<XlApi>,
        app_name: &str,
        configs: &[ChannelConfig],
    ) -> Result<Self, String> {
        if configs.is_empty() {
            return Err("No channel configurations provided".to_string());
        }

        let mode = configs[0].mode;
        let mut access_mask: XLaccess = 0;
        for cfg in configs {
            access_mask |= cfg.channel_mask;
        }

        let mut port_handle: XLportHandle = -1;
        let mut permission_mask = access_mask;

        let app_name_c =
            std::ffi::CString::new(app_name).map_err(|e| format!("Invalid app name: {}", e))?;

        let interface_version = match mode {
            CanMode::Can => 3u32,    // XL_INTERFACE_VERSION
            CanMode::CanFd => 4u32,  // XL_INTERFACE_VERSION_V4
        };

        let status = unsafe {
            (api.xl_open_port)(
                &mut port_handle,
                app_name_c.as_ptr(),
                access_mask,
                &mut permission_mask,
                8192, // rx queue size
                interface_version,
                XL_BUS_TYPE_CAN,
            )
        };

        if status != XL_SUCCESS {
            return Err(format!(
                "xlOpenPort failed: {} ({})",
                xl_status_to_string(status),
                status
            ));
        }

        // Configure bitrate for each channel
        for cfg in configs {
            if permission_mask & cfg.channel_mask != 0 {
                match cfg.mode {
                    CanMode::Can => {
                        let s = unsafe {
                            (api.xl_can_set_channel_bitrate)(
                                port_handle,
                                cfg.channel_mask,
                                cfg.bitrate,
                            )
                        };
                        if s != XL_SUCCESS {
                            log::warn!(
                                "xlCanSetChannelBitrate failed: {} ({})",
                                xl_status_to_string(s),
                                s
                            );
                        }
                    }
                    CanMode::CanFd => {
                        let fd_conf = XLcanFdConf {
                            arbitrationBitRate: cfg.bitrate,
                            sjwAbr: 0,
                            tseg1Abr: 0,
                            tseg2Abr: 0,
                            reserved1: 0,
                            dataBitRate: cfg.data_bitrate,
                            sjwDbr: 0,
                            tseg1Dbr: 0,
                            tseg2Dbr: 0,
                            reserved2: 0,
                            reserved3: 0,
                            options: 0,
                            reserved4: [0; 2],
                            reserved5: [0; 4],
                        };
                        let s = unsafe {
                            (api.xl_can_fd_set_configuration)(
                                port_handle,
                                cfg.channel_mask,
                                &fd_conf,
                            )
                        };
                        if s != XL_SUCCESS {
                            log::warn!(
                                "xlCanFdSetConfiguration failed: {} ({})",
                                xl_status_to_string(s),
                                s
                            );
                        }
                    }
                }
            }
        }

        // Set notification
        let mut notification_handle: XLhandle = 0;
        let s =
            unsafe { (api.xl_set_notification)(port_handle, &mut notification_handle, 1) };
        if s != XL_SUCCESS {
            log::warn!(
                "xlSetNotification failed: {} ({})",
                xl_status_to_string(s),
                s
            );
        }

        // Flush receive queue
        unsafe {
            (api.xl_flush_receive_queue)(port_handle);
        }

        Ok(Self {
            api,
            port_handle,
            permission_mask,
            access_mask,
            notification_handle,
            mode,
            is_active: false,
        })
    }

    /// Activate the channels
    pub fn activate(&mut self) -> Result<(), String> {
        let status = unsafe {
            (self.api.xl_activate_channel)(
                self.port_handle,
                self.access_mask,
                XL_BUS_TYPE_CAN,
                XL_ACTIVATE_RESET_CLOCK,
            )
        };
        if status != XL_SUCCESS {
            return Err(format!(
                "xlActivateChannel failed: {} ({})",
                xl_status_to_string(status),
                status
            ));
        }
        // Reset clock for synchronized timestamps
        unsafe {
            (self.api.xl_reset_clock)(self.port_handle);
        }
        self.is_active = true;
        Ok(())
    }

    /// Deactivate the channels
    pub fn deactivate(&mut self) {
        if self.is_active {
            unsafe {
                (self.api.xl_deactivate_channel)(self.port_handle, self.access_mask);
            }
            self.is_active = false;
        }
    }

    /// Transmit a classic CAN frame
    pub fn transmit_can(&self, channel_mask: XLaccess, frame: &CanFrame) -> Result<(), String> {
        let mut event = XLevent::default();
        event.tag = XL_TRANSMIT_MSG;

        let msg = unsafe { &mut event.tagData.msg };
        msg.id = frame.id;
        if frame.is_extended {
            msg.id |= 0x80000000; // EXT flag
        }
        msg.dlc = frame.dlc as u16;
        let len = frame.data.len().min(8);
        msg.data[..len].copy_from_slice(&frame.data[..len]);

        let mut msg_count = 1u32;
        let status = unsafe {
            (self.api.xl_can_transmit)(
                self.port_handle,
                channel_mask,
                &mut msg_count,
                &mut event,
            )
        };
        if status != XL_SUCCESS {
            return Err(format!(
                "xlCanTransmit failed: {} ({})",
                xl_status_to_string(status),
                status
            ));
        }
        Ok(())
    }

    /// Transmit a CAN FD frame
    pub fn transmit_canfd(&self, channel_mask: XLaccess, frame: &CanFrame) -> Result<(), String> {
        let mut msg = XLcanFdTxMsg::default();
        msg.canId = frame.id;
        if frame.is_extended {
            msg.canId |= 0x80000000;
        }
        msg.msgFlags = XL_CAN_TXMSG_FLAG_EDL;
        if frame.is_brs {
            msg.msgFlags |= XL_CAN_TXMSG_FLAG_BRS;
        }
        msg.dlc = frame.dlc;
        let len = frame.data.len().min(64);
        msg.data[..len].copy_from_slice(&frame.data[..len]);

        let mut msg_count_sent = 0u32;
        let status = unsafe {
            (self.api.xl_can_transmit_ex)(
                self.port_handle,
                channel_mask,
                1,
                &mut msg_count_sent,
                &mut msg,
            )
        };
        if status != XL_SUCCESS {
            return Err(format!(
                "xlCanTransmitEx failed: {} ({})",
                xl_status_to_string(status),
                status
            ));
        }
        Ok(())
    }

    /// Transmit a frame (auto-selects CAN or CAN-FD based on frame)
    pub fn transmit(&self, channel_mask: XLaccess, frame: &CanFrame) -> Result<(), String> {
        if frame.is_fd {
            self.transmit_canfd(channel_mask, frame)
        } else {
            self.transmit_can(channel_mask, frame)
        }
    }

    /// Receive classic CAN events (non-blocking)
    pub fn receive_can(&self) -> Result<Vec<CanFrame>, ()> {
        let mut frames = Vec::new();
        loop {
            let mut event = XLevent::default();
            let mut event_count = 1u32;
            let status =
                unsafe { (self.api.xl_receive)(self.port_handle, &mut event_count, &mut event) };
            if status == XL_ERR_QUEUE_IS_EMPTY {
                break;
            }
            if status != XL_SUCCESS {
                break;
            }
            if event.tag == XL_RECEIVE_MSG {
                let msg = unsafe { &event.tagData.msg };
                // Skip error frames
                if msg.flags as u32 & XL_CAN_MSG_FLAG_ERROR_FRAME != 0 {
                    continue;
                }
                let is_tx = msg.flags as u32 & XL_CAN_MSG_FLAG_TX_COMPLETED != 0;
                let direction = if is_tx { Direction::Tx } else { Direction::Rx };
                let is_extended = msg.id & 0x80000000 != 0;
                let id = msg.id & 0x1FFFFFFF;
                let dlc = msg.dlc as u8;
                let len = dlc.min(8) as usize;
                let timestamp = event.timeStamp as f64 / 1_000_000_000.0; // ns to seconds

                frames.push(CanFrame::new_can(
                    timestamp,
                    event.chanIndex,
                    id,
                    is_extended,
                    dlc,
                    &msg.data[..len],
                    direction,
                ));
            }
        }
        Ok(frames)
    }

    /// Receive CAN FD events (non-blocking)
    pub fn receive_canfd(&self) -> Result<Vec<CanFrame>, ()> {
        let mut frames = Vec::new();
        loop {
            let mut event = XLcanFdEvent::default();
            let status =
                unsafe { (self.api.xl_can_receive)(self.port_handle, &mut event) };
            if status == XL_ERR_QUEUE_IS_EMPTY {
                break;
            }
            if status != XL_SUCCESS {
                break;
            }
            match event.tag {
                XL_CAN_EV_TAG_RX_OK | XL_CAN_EV_TAG_TX_OK => {
                    let msg = unsafe { &event.tagData.canRxOkMsg };
                    let is_tx = event.tag == XL_CAN_EV_TAG_TX_OK;
                    let direction = if is_tx { Direction::Tx } else { Direction::Rx };
                    let is_extended = msg.id & 0x80000000 != 0;
                    let id = msg.id & 0x1FFFFFFF;
                    let is_fd = msg.flags & XL_CAN_RXMSG_FLAG_EDL != 0;
                    let is_brs = msg.flags & XL_CAN_RXMSG_FLAG_BRS != 0;
                    let dlc = msg.dlc;
                    let len = dlc_to_len(dlc);
                    let timestamp = event.timeStamp as f64 / 1_000_000_000.0;

                    if is_fd {
                        frames.push(CanFrame::new_canfd(
                            timestamp,
                            event.chanIndex,
                            id,
                            is_extended,
                            is_brs,
                            dlc,
                            &msg.data[..len],
                            direction,
                        ));
                    } else {
                        frames.push(CanFrame::new_can(
                            timestamp,
                            event.chanIndex,
                            id,
                            is_extended,
                            dlc,
                            &msg.data[..len.min(8)],
                            direction,
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(frames)
    }

    /// Receive frames based on configured mode
    pub fn receive(&self) -> Result<Vec<CanFrame>, ()> {
        match self.mode {
            CanMode::Can => self.receive_can(),
            CanMode::CanFd => self.receive_canfd(),
        }
    }

    pub fn port_handle(&self) -> XLportHandle {
        self.port_handle
    }

    pub fn notification_handle(&self) -> XLhandle {
        self.notification_handle
    }

    pub fn access_mask(&self) -> XLaccess {
        self.access_mask
    }

    pub fn permission_mask(&self) -> XLaccess {
        self.permission_mask
    }

    pub fn mode(&self) -> CanMode {
        self.mode
    }
}

impl Drop for CanPort {
    fn drop(&mut self) {
        self.deactivate();
        unsafe {
            (self.api.xl_close_port)(self.port_handle);
        }
    }
}
