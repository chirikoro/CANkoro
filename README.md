# CANkoro - CAN/CAN-FD Analysis Tool

A high-performance CAN/CAN-FD analysis application for Vector VN1630/VN1640 interfaces, built with Rust.

## Features

### CAN Communication
- **CAN / CAN-FD** send and receive via Vector XL Driver Library
- Supports **VN1630, VN1640, Virtual CAN**, and other Vector-compatible interfaces
- Multi-channel support (all channels available on the connected interface)
- Configurable bitrate and data bitrate for CAN FD

### Signal Graph Analysis
- Real-time signal plotting with DBC-based physical values
- **Independent Y-axis** per signal, shared time axis
- **Cursor analysis** - vertical cursor line across all signals showing interpolated values
- **Differential cursor analysis** - two cursors with delta value (dV), delta time (dt), and frequency calculation
- **Log file playback** mode with adjustable speed
- Data downsampling for high-performance rendering

### CAN Transmission
- **Forward mode** - Receive CAN on one channel, forward to another channel instantly
- **Modified forward** - Forward with signal value modification (physical value input, respects DBC min/max limits)
- **Trapezoidal wave** - Generate signal patterns: rise from initial value to max at specified rate, hold, then return to initial value. Works independently without receiving frames
- Save/load transmission configurations as JSON

### Custom TX Panels
User-creatable transmission control panels with the following widgets:
- **Switch** - Toggle with ON/OFF signal actions
- **ValueBox** - Numeric input linked to a signal (with min/max from DBC)
- **SelectBox** - Dropdown selection mapped to signal values
- **Label** - Display text

### Logging
- **ASC format** - Vector standard ASCII log format
- **BLF format** - Vector standard binary log format (with zlib compression)
- Real-time CAN log display with filtering (by ID, channel)

### Internationalization
- **Japanese** (default) and **English** UI
- Switch language at runtime from the top menu bar

## Requirements

### System
- **OS**: Windows 10/11 (64-bit)
- **Vector Driver**: XL Driver Library (vxlapi64.dll) must be installed
- **Hardware**: Vector VN1630, VN1640, or Virtual CAN channel

### Build
- Rust 1.75 or later
- MSVC toolchain (for Windows builds)

## Vector Hardware Setup

### 1. Install Vector Driver Setup

1. Download and install **Vector Driver Setup** from the Vector website or included media
2. Follow the installer instructions to install the XL Driver Library
3. Verify that `C:\Windows\System32\vxlapi64.dll` exists after installation

### 2. Connect Hardware

1. Connect VN1630 / VN1640 to a USB port on your PC
2. Wait for Windows to recognize the device and load drivers automatically
3. Confirm the device appears under "Vector Hardware" in Device Manager

### 3. Vector Hardware Config (Recommended, Optional)

> **Note**: CANkoro automatically detects all channels from the driver at startup, so pre-configuration in Hardware Config is not required. However, performing the following setup ensures more reliable channel assignment.

1. Launch **Vector Hardware Config** from the Start menu
2. Verify your connected hardware (VN1630/VN1640) appears in the device list
3. (Optional) Register the application:
   - In the **Application** section, click **Add**
   - Enter `CANkoro` as the application name
   - Assign the channels you want to use (Channel 1, Channel 2, etc.)
   - Set **Bus Type** to `CAN`
4. (Optional) Virtual CAN setup:
   - In the **Virtual Devices** tab, you can add virtual CAN channels
   - This allows testing the application without physical hardware

### 4. Virtual CAN (Using Without Hardware)

You can use Vector's Virtual CAN channels to test CANkoro without physical hardware:

1. Launch **Vector Hardware Config**
2. Go to the **Virtual Devices** tab
3. Click **Add** to add virtual CAN channels (2 or more recommended)
4. Launch CANkoro - channels marked `[Virtual]` will appear in the channel config
5. Forwarding test: You can test forwarding mode between Virtual Channel 1 → Virtual Channel 2

### Connection Flow (Summary)

```
Install Vector Driver Setup
        ↓
Connect VN1630/VN1640 via USB (or configure Virtual CAN)
        ↓
Launch CANkoro
        ↓
Channel Config tab: channels are auto-detected
  → [VN1630] Channel 1 (S/N: xxxxx, Ch: 0)
  → [VN1630] Channel 2 (S/N: xxxxx, Ch: 1)
  → [Virtual] Virtual Channel 1 ...
        ↓
Check channels to use → set mode/bitrate
        ↓
Click "Connect" → CAN communication starts
```

## Build

```bash
# Debug build
cargo build

# Release build (optimized, LTO enabled)
cargo build --release
```

The release binary is located at `target/release/cankoro.exe`.

> Note: In release mode, the console window is automatically hidden (`windows_subsystem = "windows"`). In debug mode, the console remains visible for log output.

## Usage

### 1. Channel Configuration
1. Launch CANkoro
2. Go to **Channel Config** tab
3. Select a DBC file (optional, required for signal-based features)
4. Enable desired channels and configure bitrate/mode (CAN or CAN FD)
5. Click **Connect**

### 2. Viewing CAN Log
- Switch to the **CAN Log** tab
- Use the ID filter to search for specific message IDs (hex format)
- Toggle auto-scroll and pause/resume as needed

### 3. Signal Graph
- Switch to the **Graph** tab
- Click **Select Signals** to choose which signals to display
- Use **C1** checkbox to enable cursor analysis
- Enable **Diff** and **C2** for differential cursor analysis
- For log playback: switch to Playback mode, load a log file, and press Play

### 4. CAN Transmission
- **TX Config** tab: Configure forwarding rules and trapezoidal wave patterns
- **TX Panels** tab: Use custom control panels to send signals interactively
- **Panel Editor** tab: Create and customize transmission panels

## Project Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # Application state and orchestration
├── i18n/                # Internationalization (Japanese/English)
├── vector/              # Vector XL Driver FFI bindings
│   ├── xlapi.rs         # Raw FFI function declarations
│   ├── types.rs         # XL type definitions
│   ├── driver.rs        # Driver wrapper (open/close/config)
│   └── channel.rs       # Channel management, CAN port
├── can/                 # CAN frame types and processing
│   ├── frame.rs         # CAN frame definition
│   ├── receiver.rs      # Receive thread
│   └── transmitter.rs   # Transmit thread (forward/trapezoidal)
├── dbc/                 # DBC file parser
│   ├── parser.rs        # nom-based DBC parser
│   ├── database.rs      # DBC database
│   ├── message.rs       # Message definition
│   └── signal.rs        # Signal definition and conversion
├── log/                 # Log file I/O
│   ├── asc.rs           # ASC writer
│   ├── blf.rs           # BLF writer
│   └── reader.rs        # Log file reader (ASC/BLF)
├── graph/               # Graph plotting
│   ├── plot.rs          # Signal plot data
│   ├── cursor.rs        # Cursor management
│   └── timeline.rs      # Time axis control
├── ui/                  # UI components
│   ├── main_view.rs     # Tab bar and status bar
│   ├── channel_config.rs
│   ├── log_view.rs
│   ├── graph_view.rs
│   ├── tx_config.rs
│   ├── tx_panel.rs
│   └── panel_editor.rs
└── config/              # Configuration
    └── tx_settings.rs   # TX settings save/load (JSON)
```

## Architecture

```
┌─────────────────┐     crossbeam       ┌──────────────────┐
│  CAN RX Thread  │ ──────────────────> │   Main Thread    │
│  (per channel)  │      CanFrame       │   (egui GUI)     │
└─────────────────┘                     └──────────────────┘
                                               │
┌─────────────────┐     crossbeam       ┌──────────────────┐
│  CAN TX Thread  │ <────────────────── │  Log Writer      │
│  (fwd/trapez.)  │     TxCommand       │  (ASC/BLF)       │
└─────────────────┘                     └──────────────────┘
```

- **CAN RX Thread**: Blocking receive via `xlReceive`, sends frames through lock-free channel
- **CAN TX Thread**: Handles forwarding and trapezoidal wave generation
- **Main Thread**: egui GUI rendering at 60fps
- **Log Writer**: Asynchronous buffered file writes

## Performance Optimizations
- Lock-free inter-thread communication (crossbeam-channel)
- Fast mutex (parking_lot)
- Immediate-mode GUI rendering (egui)
- Data downsampling for large datasets in graph display
- Release build with LTO and single codegen unit

## License

All rights reserved.
