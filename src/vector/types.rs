/// Vector XL Driver Library type definitions
/// Mirrors the C types from vxlapi.h

pub type XLstatus = i32;
pub type XLportHandle = i64;
pub type XLaccess = u64;
pub type XLhandle = u64;

// Status codes
pub const XL_SUCCESS: XLstatus = 0;
pub const XL_ERR_QUEUE_IS_EMPTY: XLstatus = 10;
pub const XL_ERR_QUEUE_IS_FULL: XLstatus = 11;
pub const XL_ERR_INVALID_CHAN_INDEX: XLstatus = 20;
pub const XL_ERR_INVALID_ACCESS: XLstatus = 21;
pub const XL_ERR_PORT_IS_OFFLINE: XLstatus = 22;
pub const XL_ERR_CHAN_IS_ONLINE: XLstatus = 23;
pub const XL_ERR_NOT_IMPLEMENTED: XLstatus = 32;
pub const XL_ERR_INVALID_PORT: XLstatus = 33;
pub const XL_ERR_HW_NOT_PRESENT: XLstatus = 129;
pub const XL_ERR_NOTIFY_ALREADY_ACTIVE: XLstatus = 131;
pub const XL_ERR_NO_RESOURCES: XLstatus = 152;
pub const XL_ERR_WRONG_CHIP_TYPE: XLstatus = 153;
pub const XL_ERR_WRONG_COMMAND: XLstatus = 154;
pub const XL_ERR_INVALID_HANDLE: XLstatus = 155;
pub const XL_ERR_CANNOT_OPEN_DRIVER: XLstatus = 201;

// Bus types
pub const XL_BUS_TYPE_CAN: u32 = 0x00000001;

// Interface types
pub const XL_HWTYPE_VIRTUAL: u32 = 1;
pub const XL_HWTYPE_CANCARDXL: u32 = 15;
pub const XL_HWTYPE_VN1610: u32 = 50;
pub const XL_HWTYPE_VN1630: u32 = 57;
pub const XL_HWTYPE_VN1640: u32 = 59;
pub const XL_HWTYPE_VN1670: u32 = 137;

// CAN message flags
pub const XL_CAN_MSG_FLAG_ERROR_FRAME: u32 = 0x01;
pub const XL_CAN_MSG_FLAG_OVERRUN: u32 = 0x02;
pub const XL_CAN_MSG_FLAG_NERR: u32 = 0x04;
pub const XL_CAN_MSG_FLAG_WAKEUP: u32 = 0x08;
pub const XL_CAN_MSG_FLAG_REMOTE_FRAME: u32 = 0x10;
pub const XL_CAN_MSG_FLAG_RESERVED_1: u32 = 0x20;
pub const XL_CAN_MSG_FLAG_TX_COMPLETED: u32 = 0x40;
pub const XL_CAN_MSG_FLAG_TX_REQUEST: u32 = 0x80;

// CAN FD flags
pub const XL_CAN_TXMSG_FLAG_EDL: u32 = 0x0001; // Extended Data Length (FD)
pub const XL_CAN_TXMSG_FLAG_BRS: u32 = 0x0002; // Bit Rate Switch
pub const XL_CAN_TXMSG_FLAG_RTR: u32 = 0x0010; // Remote Transmission Request
pub const XL_CAN_TXMSG_FLAG_HIGHPRIO: u32 = 0x0080;

pub const XL_CAN_RXMSG_FLAG_EDL: u32 = 0x0001;
pub const XL_CAN_RXMSG_FLAG_BRS: u32 = 0x0002;
pub const XL_CAN_RXMSG_FLAG_ESI: u32 = 0x0004;
pub const XL_CAN_RXMSG_FLAG_EF: u32 = 0x0200;

// XL event tags
pub const XL_RECEIVE_MSG: u16 = 1;
pub const XL_CHIP_STATE: u16 = 4;
pub const XL_TRANSCEIVER: u16 = 6;
pub const XL_TIMER: u16 = 8;
pub const XL_TRANSMIT_MSG: u16 = 10;

// CAN FD event tags
pub const XL_CAN_EV_TAG_RX_OK: u16 = 0x0400;
pub const XL_CAN_EV_TAG_TX_OK: u16 = 0x0404;
pub const XL_CAN_EV_TAG_TX_ERROR: u16 = 0x040C;
pub const XL_CAN_EV_TAG_CHIP_STATE: u16 = 0x0409;

// Activate flags
pub const XL_ACTIVATE_NONE: u32 = 0;
pub const XL_ACTIVATE_RESET_CLOCK: u32 = 8;

// Max constants
pub const XL_MAX_APPNAME: usize = 32;
pub const XL_MAX_MSG_LEN: usize = 8;
pub const XL_CAN_MAX_DATA_LEN: usize = 64;
pub const XL_CONFIG_MAX_CHANNELS: usize = 64;

/// Driver configuration for a single channel
#[repr(C)]
#[derive(Debug, Clone)]
pub struct XLchannelConfig {
    pub name: [u8; 32],
    pub hw_type: u32,
    pub hw_index: u32,
    pub hw_channel: u32,
    pub transceiverType: u32,
    pub transceiverState: u32,
    pub configError: u32,
    pub channelIndex: u32,
    pub channelMask: XLaccess,
    pub channelCapabilities: u32,
    pub channelBusCapabilities: u32,
    pub isOnBus: u32,
    pub connectedBusType: u32,
    pub busParams: XLbusParams,
    pub _doNotUse: u32,
    pub driverVersion: u32,
    pub interfaceVersion: u32,
    pub raw_data: [u32; 10],
    pub serialNumber: u32,
    pub articleNumber: u32,
    pub transceiverName: [u8; 32],
    pub specialCabFlags: u32,
    pub dominantTimeout: u32,
    pub dominantRecessiveDelay: u8,
    pub recessiveDominantDelay: u8,
    pub connectionInfo: u8,
    pub currentlyAvailableTimestamps: u8,
    pub minimalSupplyVoltage: u16,
    pub maximalSupplyVoltage: u16,
    pub maximalBaudrate: u32,
    pub fpgaCoreCapabilities: u8,
    pub specialDeviceStatus: u8,
    pub channelBusActiveCapabilities: u16,
    pub breakOffset: u16,
    pub delimiterOffset: u16,
    pub reserved: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XLbusParams {
    pub busType: u32,
    pub data: XLbusParamsData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union XLbusParamsData {
    pub can: XLbusParamsCan,
    pub raw: [u8; 32],
}

impl std::fmt::Debug for XLbusParamsData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XLbusParamsData {{ ... }}")
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XLbusParamsCan {
    pub bitRate: u32,
    pub sjw: u8,
    pub tseg1: u8,
    pub tseg2: u8,
    pub sam: u8,
    pub outputMode: u8,
    pub reserved: [u8; 7],
    pub canOpMode: u8,
}

/// Driver configuration
#[repr(C)]
#[derive(Debug, Clone)]
pub struct XLdriverConfig {
    pub dllVersion: u32,
    pub channelCount: u32,
    pub reserved: [u32; 10],
    pub channel: [XLchannelConfig; XL_CONFIG_MAX_CHANNELS],
}

/// CAN message structure (Classic CAN)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XLcanMsg {
    pub id: u32,
    pub flags: u16,
    pub dlc: u16,
    pub res1: u64,
    pub data: [u8; XL_MAX_MSG_LEN],
    pub res2: u64,
}

/// XL event structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XLevent {
    pub tag: u16,
    pub chanIndex: u8,
    pub transId: u8,
    pub portHandle: u16,
    pub flags: u8,
    pub reserved: u8,
    pub timeStamp: u64,
    pub tagData: XLeventData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union XLeventData {
    pub msg: XLcanMsg,
    pub raw: [u8; 46],
}

impl std::fmt::Debug for XLeventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XLeventData {{ ... }}")
    }
}

/// CAN FD message (RX)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XLcanFdRxMsg {
    pub id: u32,
    pub flags: u32,
    pub dlc: u8,
    pub reserved1: [u8; 3],
    pub totalBitCnt: u16,
    pub reserved2: [u8; 2],
    pub data: [u8; XL_CAN_MAX_DATA_LEN],
}

/// CAN FD message (TX)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XLcanFdTxMsg {
    pub canId: u32,
    pub msgFlags: u32,
    pub dlc: u8,
    pub reserved1: [u8; 7],
    pub data: [u8; XL_CAN_MAX_DATA_LEN],
}

/// CAN FD event
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XLcanFdEvent {
    pub tag: u16,
    pub chanIndex: u8,
    pub reserved1: [u8; 1],
    pub userHandle: u32,
    pub flagsChip: u16,
    pub reserved2: [u8; 2],
    pub timeStamp: u64,
    pub tagData: XLcanFdEventData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union XLcanFdEventData {
    pub canRxOkMsg: XLcanFdRxMsg,
    pub canTxOkMsg: XLcanFdRxMsg,
    pub raw: [u8; 200],
}

impl std::fmt::Debug for XLcanFdEventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XLcanFdEventData {{ ... }}")
    }
}

/// CAN FD configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct XLcanFdConf {
    pub arbitrationBitRate: u32,
    pub sjwAbr: u8,
    pub tseg1Abr: u8,
    pub tseg2Abr: u8,
    pub reserved1: u8,
    pub dataBitRate: u32,
    pub sjwDbr: u8,
    pub tseg1Dbr: u8,
    pub tseg2Dbr: u8,
    pub reserved2: u8,
    pub reserved3: u8,
    pub options: u8,
    pub reserved4: [u8; 2],
    pub reserved5: [u32; 4],
}

impl Default for XLchannelConfig {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLdriverConfig {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLcanMsg {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLevent {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLcanFdTxMsg {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLcanFdConf {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLcanFdEvent {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Default for XLbusParams {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

/// Helper to convert DLC to byte count (CAN FD)
pub fn dlc_to_len(dlc: u8) -> usize {
    match dlc {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 12,
        10 => 16,
        11 => 20,
        12 => 24,
        13 => 32,
        14 => 48,
        15 => 64,
        _ => 8,
    }
}

/// Helper to convert byte count to DLC (CAN FD)
pub fn len_to_dlc(len: usize) -> u8 {
    match len {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9..=12 => 9,
        13..=16 => 10,
        17..=20 => 11,
        21..=24 => 12,
        25..=32 => 13,
        33..=48 => 14,
        49..=64 => 15,
        _ => 15,
    }
}

pub fn xl_status_to_string(status: XLstatus) -> &'static str {
    match status {
        XL_SUCCESS => "XL_SUCCESS",
        XL_ERR_QUEUE_IS_EMPTY => "XL_ERR_QUEUE_IS_EMPTY",
        XL_ERR_QUEUE_IS_FULL => "XL_ERR_QUEUE_IS_FULL",
        XL_ERR_INVALID_CHAN_INDEX => "XL_ERR_INVALID_CHAN_INDEX",
        XL_ERR_INVALID_ACCESS => "XL_ERR_INVALID_ACCESS",
        XL_ERR_PORT_IS_OFFLINE => "XL_ERR_PORT_IS_OFFLINE",
        XL_ERR_CHAN_IS_ONLINE => "XL_ERR_CHAN_IS_ONLINE",
        XL_ERR_NOT_IMPLEMENTED => "XL_ERR_NOT_IMPLEMENTED",
        XL_ERR_INVALID_PORT => "XL_ERR_INVALID_PORT",
        XL_ERR_HW_NOT_PRESENT => "XL_ERR_HW_NOT_PRESENT",
        XL_ERR_NO_RESOURCES => "XL_ERR_NO_RESOURCES",
        XL_ERR_WRONG_CHIP_TYPE => "XL_ERR_WRONG_CHIP_TYPE",
        XL_ERR_WRONG_COMMAND => "XL_ERR_WRONG_COMMAND",
        XL_ERR_INVALID_HANDLE => "XL_ERR_INVALID_HANDLE",
        XL_ERR_CANNOT_OPEN_DRIVER => "XL_ERR_CANNOT_OPEN_DRIVER",
        _ => "XL_ERR_UNKNOWN",
    }
}
