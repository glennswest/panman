use panman_core::error::Result;

/// Display buffer callback type for LVGL integration.
pub type FlushCb = Box<dyn FnMut(&[u8], u16, u16, u16, u16) + Send>;

/// Touch input reading.
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressed: bool,
}

/// Peripherals returned by board initialization.
pub struct BoardPeripherals {
    /// Display width in pixels.
    pub width: u16,
    /// Display height in pixels.
    pub height: u16,
}

/// Hardware abstraction trait — one implementation per panel board.
///
/// Each board implementation handles:
/// - Display initialization (DSI, SPI, parallel, etc.)
/// - Touch controller initialization
/// - Backlight control
/// - WiFi initialization (if board-specific)
///
/// On ESP32, implementations use `esp-idf-svc` and board-specific BSP calls.
/// The trait is defined here without ESP32 dependencies so the workspace
/// compiles on any host for development and testing.
pub trait Board {
    /// Human-readable board name.
    fn name() -> &'static str;

    /// Native display resolution (width, height).
    fn resolution() -> (u16, u16);

    /// Initialize all board peripherals.
    fn init() -> Result<BoardPeripherals>;

    /// Set backlight brightness (0-100).
    fn set_backlight(percent: u8) -> Result<()>;

    /// Read current touch state.
    fn read_touch() -> Result<Option<TouchPoint>>;
}
