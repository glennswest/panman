use crate::board::{Board, BoardPeripherals, TouchPoint};
use panman_core::error::Result;

/// CrowPanel Advanced 10.1" ESP32-P4
///
/// Hardware specs:
/// - SoC: ESP32-P4 (RISC-V dual-core, 400MHz)
/// - Display: 10.1" 1024x600 IPS, MIPI DSI interface
/// - Touch: Goodix GT911 capacitive, I2C
/// - Backlight: PWM-controlled LED
/// - WiFi: via ESP32-C6 companion chip (SPI)
/// - Flash: 16MB, PSRAM: 32MB LPSRAM
/// - USB: Type-C (CDC/JTAG)
///
/// On actual ESP32 hardware, `init()` will:
/// 1. Configure MIPI DSI with 2-lane, 1024x600, RGB888
/// 2. Initialize GT911 touch via I2C (SDA/SCL GPIOs from BSP)
/// 3. Set up PWM channel for backlight LED
/// 4. Initialize WiFi via ESP32-C6 companion SPI bridge
///
/// This stub compiles on any host. The real implementation lives behind
/// `#[cfg(target_os = "espidf")]` blocks in the board binary crate.
pub struct CrowPanelP4_10;

impl Board for CrowPanelP4_10 {
    fn name() -> &'static str {
        "CrowPanel Advanced 10.1\" ESP32-P4"
    }

    fn resolution() -> (u16, u16) {
        (1024, 600)
    }

    fn init() -> Result<BoardPeripherals> {
        log::info!("Initializing {}", Self::name());

        // On ESP32-P4, this would:
        // 1. esp_lcd_new_panel_dsi() for MIPI DSI display
        // 2. esp_lcd_touch_new_i2c_gt911() for touch
        // 3. ledc_timer_config() + ledc_channel_config() for backlight
        // Real init happens in boards/crowpanel-p4-10/src/main.rs using esp-idf-svc

        let (width, height) = Self::resolution();
        Ok(BoardPeripherals { width, height })
    }

    fn set_backlight(percent: u8) -> Result<()> {
        let duty = percent.min(100);
        log::debug!("Backlight: {}%", duty);
        // On ESP32: ledc_set_duty() + ledc_update_duty()
        Ok(())
    }

    fn read_touch() -> Result<Option<TouchPoint>> {
        // On ESP32: esp_lcd_touch_read_data() -> TouchPoint
        Ok(None)
    }
}
