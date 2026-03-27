/// Vector XL Driver Library FFI bindings
/// Runtime dynamic loading of vxlapi64.dll / vxlapi.dll
use super::types::*;

#[cfg(target_os = "windows")]
use std::ffi::CString;

/// Function pointer types matching the vxlapi C API
type FnXlOpenDriver = unsafe extern "C" fn() -> XLstatus;
type FnXlCloseDriver = unsafe extern "C" fn() -> XLstatus;
type FnXlGetDriverConfig = unsafe extern "C" fn(pDriverConfig: *mut XLdriverConfig) -> XLstatus;
type FnXlGetApplConfig = unsafe extern "C" fn(
    appName: *const i8,
    appChannel: u32,
    pHwType: *mut u32,
    pHwIndex: *mut u32,
    pHwChannel: *mut u32,
    busType: u32,
) -> XLstatus;
type FnXlSetApplConfig = unsafe extern "C" fn(
    appName: *const i8,
    appChannel: u32,
    hwType: u32,
    hwIndex: u32,
    hwChannel: u32,
    busType: u32,
) -> XLstatus;
type FnXlGetChannelIndex = unsafe extern "C" fn(hwType: i32, hwIndex: i32, hwChannel: i32) -> i32;
type FnXlGetChannelMask =
    unsafe extern "C" fn(hwType: i32, hwIndex: i32, hwChannel: i32) -> XLaccess;
type FnXlOpenPort = unsafe extern "C" fn(
    pPortHandle: *mut XLportHandle,
    userName: *const i8,
    accessMask: XLaccess,
    pPermissionMask: *mut XLaccess,
    rxQueueSize: u32,
    xlInterfaceVersion: u32,
    busType: u32,
) -> XLstatus;
type FnXlClosePort = unsafe extern "C" fn(portHandle: XLportHandle) -> XLstatus;
type FnXlActivateChannel = unsafe extern "C" fn(
    portHandle: XLportHandle,
    accessMask: XLaccess,
    busType: u32,
    flags: u32,
) -> XLstatus;
type FnXlDeactivateChannel =
    unsafe extern "C" fn(portHandle: XLportHandle, accessMask: XLaccess) -> XLstatus;
type FnXlCanSetChannelBitrate =
    unsafe extern "C" fn(portHandle: XLportHandle, accessMask: XLaccess, bitrate: u32)
        -> XLstatus;
type FnXlCanFdSetConfiguration = unsafe extern "C" fn(
    portHandle: XLportHandle,
    accessMask: XLaccess,
    pCanFdConf: *const XLcanFdConf,
) -> XLstatus;
type FnXlCanTransmit = unsafe extern "C" fn(
    portHandle: XLportHandle,
    accessMask: XLaccess,
    messageCount: *mut u32,
    pMessages: *mut XLevent,
) -> XLstatus;
type FnXlCanTransmitEx = unsafe extern "C" fn(
    portHandle: XLportHandle,
    accessMask: XLaccess,
    msgCnt: u32,
    pMsgCntSent: *mut u32,
    pMessages: *mut XLcanFdTxMsg,
) -> XLstatus;
type FnXlReceive =
    unsafe extern "C" fn(portHandle: XLportHandle, pEventCount: *mut u32, pEvent: *mut XLevent)
        -> XLstatus;
type FnXlCanReceive = unsafe extern "C" fn(
    portHandle: XLportHandle,
    pXlCanRxEvt: *mut XLcanFdEvent,
) -> XLstatus;
type FnXlSetNotification = unsafe extern "C" fn(
    portHandle: XLportHandle,
    pHandle: *mut XLhandle,
    qualifiedSize: i32,
) -> XLstatus;
type FnXlResetClock = unsafe extern "C" fn(portHandle: XLportHandle) -> XLstatus;
type FnXlSetTimerRate =
    unsafe extern "C" fn(portHandle: XLportHandle, timerRate: u64) -> XLstatus;
type FnXlCanSetChannelAcceptance = unsafe extern "C" fn(
    portHandle: XLportHandle,
    accessMask: XLaccess,
    code: u32,
    mask: u32,
    idRange: u32,
) -> XLstatus;
type FnXlFlushReceiveQueue = unsafe extern "C" fn(portHandle: XLportHandle) -> XLstatus;
type FnXlGetEventString = unsafe extern "C" fn(pEvent: *const XLevent) -> *const i8;
type FnXlCanSetChannelOutput =
    unsafe extern "C" fn(portHandle: XLportHandle, accessMask: XLaccess, mode: u8) -> XLstatus;

/// Holds dynamically loaded function pointers from vxlapi
pub struct XlApi {
    #[cfg(target_os = "windows")]
    _lib: windows::Win32::Foundation::HMODULE,
    #[cfg(not(target_os = "windows"))]
    _lib: usize,

    pub xl_open_driver: FnXlOpenDriver,
    pub xl_close_driver: FnXlCloseDriver,
    pub xl_get_driver_config: FnXlGetDriverConfig,
    pub xl_get_appl_config: FnXlGetApplConfig,
    pub xl_set_appl_config: FnXlSetApplConfig,
    pub xl_get_channel_index: FnXlGetChannelIndex,
    pub xl_get_channel_mask: FnXlGetChannelMask,
    pub xl_open_port: FnXlOpenPort,
    pub xl_close_port: FnXlClosePort,
    pub xl_activate_channel: FnXlActivateChannel,
    pub xl_deactivate_channel: FnXlDeactivateChannel,
    pub xl_can_set_channel_bitrate: FnXlCanSetChannelBitrate,
    pub xl_can_fd_set_configuration: FnXlCanFdSetConfiguration,
    pub xl_can_transmit: FnXlCanTransmit,
    pub xl_can_transmit_ex: FnXlCanTransmitEx,
    pub xl_receive: FnXlReceive,
    pub xl_can_receive: FnXlCanReceive,
    pub xl_set_notification: FnXlSetNotification,
    pub xl_reset_clock: FnXlResetClock,
    pub xl_set_timer_rate: FnXlSetTimerRate,
    pub xl_can_set_channel_acceptance: FnXlCanSetChannelAcceptance,
    pub xl_flush_receive_queue: FnXlFlushReceiveQueue,
    pub xl_get_event_string: FnXlGetEventString,
    pub xl_can_set_channel_output: FnXlCanSetChannelOutput,
}

unsafe impl Send for XlApi {}
unsafe impl Sync for XlApi {}

#[cfg(target_os = "windows")]
impl XlApi {
    pub fn load() -> Result<Self, String> {
        use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
        use windows::core::PCSTR;

        unsafe {
            let lib_name = PCSTR::from_raw(b"vxlapi64.dll\0".as_ptr());
            let lib = LoadLibraryA(lib_name)
                .map_err(|e| format!("Failed to load vxlapi64.dll: {}", e))?;

            macro_rules! load_fn {
                ($name:expr) => {{
                    let cname = CString::new($name).unwrap();
                    let proc = GetProcAddress(lib, PCSTR::from_raw(cname.as_ptr() as *const u8))
                        .ok_or_else(|| format!("Failed to find function: {}", $name))?;
                    std::mem::transmute(proc)
                }};
            }

            Ok(Self {
                _lib: lib,
                xl_open_driver: load_fn!("xlOpenDriver"),
                xl_close_driver: load_fn!("xlCloseDriver"),
                xl_get_driver_config: load_fn!("xlGetDriverConfig"),
                xl_get_appl_config: load_fn!("xlGetApplConfig"),
                xl_set_appl_config: load_fn!("xlSetApplConfig"),
                xl_get_channel_index: load_fn!("xlGetChannelIndex"),
                xl_get_channel_mask: load_fn!("xlGetChannelMask"),
                xl_open_port: load_fn!("xlOpenPort"),
                xl_close_port: load_fn!("xlClosePort"),
                xl_activate_channel: load_fn!("xlActivateChannel"),
                xl_deactivate_channel: load_fn!("xlDeactivateChannel"),
                xl_can_set_channel_bitrate: load_fn!("xlCanSetChannelBitrate"),
                xl_can_fd_set_configuration: load_fn!("xlCanFdSetConfiguration"),
                xl_can_transmit: load_fn!("xlCanTransmit"),
                xl_can_transmit_ex: load_fn!("xlCanTransmitEx"),
                xl_receive: load_fn!("xlReceive"),
                xl_can_receive: load_fn!("xlCanReceive"),
                xl_set_notification: load_fn!("xlSetNotification"),
                xl_reset_clock: load_fn!("xlResetClock"),
                xl_set_timer_rate: load_fn!("xlSetTimerRate"),
                xl_can_set_channel_acceptance: load_fn!("xlCanSetChannelAcceptance"),
                xl_flush_receive_queue: load_fn!("xlFlushReceiveQueue"),
                xl_get_event_string: load_fn!("xlGetEventString"),
                xl_can_set_channel_output: load_fn!("xlCanSetChannelOutput"),
            })
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl XlApi {
    /// Stub for non-Windows platforms (for development/compilation)
    pub fn load() -> Result<Self, String> {
        unsafe extern "C" fn stub_open() -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_close() -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_get_config(_: *mut XLdriverConfig) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_get_appl(_: *const i8, _: u32, _: *mut u32, _: *mut u32, _: *mut u32, _: u32) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_set_appl(_: *const i8, _: u32, _: u32, _: u32, _: u32, _: u32) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_get_ch_idx(_: i32, _: i32, _: i32) -> i32 { -1 }
        unsafe extern "C" fn stub_get_ch_mask(_: i32, _: i32, _: i32) -> XLaccess { 0 }
        unsafe extern "C" fn stub_open_port(_: *mut XLportHandle, _: *const i8, _: XLaccess, _: *mut XLaccess, _: u32, _: u32, _: u32) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_close_port(_: XLportHandle) -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_activate(_: XLportHandle, _: XLaccess, _: u32, _: u32) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_deactivate(_: XLportHandle, _: XLaccess) -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_set_bitrate(_: XLportHandle, _: XLaccess, _: u32) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_fd_config(_: XLportHandle, _: XLaccess, _: *const XLcanFdConf) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_transmit(_: XLportHandle, _: XLaccess, _: *mut u32, _: *mut XLevent) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_transmit_ex(_: XLportHandle, _: XLaccess, _: u32, _: *mut u32, _: *mut XLcanFdTxMsg) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_receive(_: XLportHandle, _: *mut u32, _: *mut XLevent) -> XLstatus { XL_ERR_QUEUE_IS_EMPTY }
        unsafe extern "C" fn stub_can_receive(_: XLportHandle, _: *mut XLcanFdEvent) -> XLstatus { XL_ERR_QUEUE_IS_EMPTY }
        unsafe extern "C" fn stub_set_notif(_: XLportHandle, _: *mut XLhandle, _: i32) -> XLstatus { XL_ERR_CANNOT_OPEN_DRIVER }
        unsafe extern "C" fn stub_reset_clock(_: XLportHandle) -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_timer_rate(_: XLportHandle, _: u64) -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_acceptance(_: XLportHandle, _: XLaccess, _: u32, _: u32, _: u32) -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_flush(_: XLportHandle) -> XLstatus { XL_SUCCESS }
        unsafe extern "C" fn stub_event_str(_: *const XLevent) -> *const i8 { std::ptr::null() }
        unsafe extern "C" fn stub_set_output(_: XLportHandle, _: XLaccess, _: u8) -> XLstatus { XL_SUCCESS }

        Ok(Self {
            _lib: 0,
            xl_open_driver: stub_open,
            xl_close_driver: stub_close,
            xl_get_driver_config: stub_get_config,
            xl_get_appl_config: stub_get_appl,
            xl_set_appl_config: stub_set_appl,
            xl_get_channel_index: stub_get_ch_idx,
            xl_get_channel_mask: stub_get_ch_mask,
            xl_open_port: stub_open_port,
            xl_close_port: stub_close_port,
            xl_activate_channel: stub_activate,
            xl_deactivate_channel: stub_deactivate,
            xl_can_set_channel_bitrate: stub_set_bitrate,
            xl_can_fd_set_configuration: stub_fd_config,
            xl_can_transmit: stub_transmit,
            xl_can_transmit_ex: stub_transmit_ex,
            xl_receive: stub_receive,
            xl_can_receive: stub_can_receive,
            xl_set_notification: stub_set_notif,
            xl_reset_clock: stub_reset_clock,
            xl_set_timer_rate: stub_timer_rate,
            xl_can_set_channel_acceptance: stub_acceptance,
            xl_flush_receive_queue: stub_flush,
            xl_get_event_string: stub_event_str,
            xl_can_set_channel_output: stub_set_output,
        })
    }
}
