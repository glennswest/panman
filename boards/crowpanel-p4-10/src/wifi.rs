//! WiFi initialization and management for ESP32-P4 with C6 companion.
//!
//! Uses `esp_hosted` (SDIO transport) + `esp_wifi_remote` for transparent
//! WiFi API access through the ESP32-C6 companion chip.
//!
//! The standard `esp_wifi_*` API calls are routed through the hosted
//! transport to the C6, which handles the actual WiFi radio.

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "espidf")]
static WIFI_CONNECTED: AtomicBool = AtomicBool::new(false);
static WIFI_GOT_IP: AtomicBool = AtomicBool::new(false);

/// Check if WiFi has obtained an IP address.
pub fn is_connected() -> bool {
    WIFI_GOT_IP.load(Ordering::Relaxed)
}

// ---- ESP-IDF target implementation ----

#[cfg(target_os = "espidf")]
use panman_core::error::PanmanError;

#[cfg(target_os = "espidf")]
fn check_esp(result: esp_idf_sys::esp_err_t, context: &str) -> panman_core::error::Result<()> {
    if result == esp_idf_sys::ESP_OK {
        Ok(())
    } else {
        Err(PanmanError::Wifi(format!("{}: error 0x{:x}", context, result)))
    }
}

/// Initialize NVS flash (required before WiFi).
#[cfg(target_os = "espidf")]
pub fn init_nvs() -> panman_core::error::Result<()> {
    unsafe {
        let ret = esp_idf_sys::nvs_flash_init();
        if ret == esp_idf_sys::ESP_ERR_NVS_NO_FREE_PAGES
            || ret == esp_idf_sys::ESP_ERR_NVS_NEW_VERSION_FOUND
        {
            check_esp(esp_idf_sys::nvs_flash_erase(), "nvs_flash_erase")?;
            check_esp(esp_idf_sys::nvs_flash_init(), "nvs_flash_init")?;
        } else {
            check_esp(ret, "nvs_flash_init")?;
        }
    }
    log::info!("NVS flash initialized");
    Ok(())
}

/// Initialize WiFi subsystem in STA mode.
///
/// This sets up the network interface, event loop, and WiFi driver.
/// The `esp_hosted` component transparently handles the SDIO transport
/// to the ESP32-C6 companion chip.
#[cfg(target_os = "espidf")]
pub fn init() -> panman_core::error::Result<()> {
    unsafe {
        // Initialize networking stack
        check_esp(esp_idf_sys::esp_netif_init(), "esp_netif_init")?;
        check_esp(
            esp_idf_sys::esp_event_loop_create_default(),
            "esp_event_loop_create_default",
        )?;

        let sta = esp_idf_sys::esp_netif_create_default_wifi_sta();
        if sta.is_null() {
            return Err(PanmanError::Wifi("failed to create WiFi STA netif".into()));
        }

        // Initialize WiFi with default config.
        // When esp_wifi_remote is active, this is routed to the C6.
        // The magic number validates the config struct.
        let mut cfg: esp_idf_sys::wifi_init_config_t = core::mem::zeroed();
        cfg.magic = esp_idf_sys::WIFI_INIT_CONFIG_MAGIC as i32;
        check_esp(esp_idf_sys::esp_wifi_init(&cfg), "esp_wifi_init")?;

        // Register WiFi event handler
        let mut instance_wifi: esp_idf_sys::esp_event_handler_instance_t = core::ptr::null_mut();
        check_esp(
            esp_idf_sys::esp_event_handler_instance_register(
                esp_idf_sys::WIFI_EVENT,
                esp_idf_sys::ESP_EVENT_ANY_ID,
                Some(wifi_event_handler),
                core::ptr::null_mut(),
                &mut instance_wifi,
            ),
            "register WIFI_EVENT handler",
        )?;

        // Register IP event handler
        let mut instance_ip: esp_idf_sys::esp_event_handler_instance_t = core::ptr::null_mut();
        check_esp(
            esp_idf_sys::esp_event_handler_instance_register(
                esp_idf_sys::IP_EVENT,
                esp_idf_sys::ip_event_t_IP_EVENT_STA_GOT_IP as i32,
                Some(ip_event_handler),
                core::ptr::null_mut(),
                &mut instance_ip,
            ),
            "register IP_EVENT handler",
        )?;

        // Set STA mode and start WiFi
        check_esp(
            esp_idf_sys::esp_wifi_set_mode(esp_idf_sys::wifi_mode_t_WIFI_MODE_STA),
            "esp_wifi_set_mode",
        )?;
        check_esp(esp_idf_sys::esp_wifi_start(), "esp_wifi_start")?;
    }
    log::info!("WiFi initialized in STA mode");
    Ok(())
}

/// Connect to a WiFi access point.
#[cfg(target_os = "espidf")]
pub fn connect(ssid: &str, password: &str) -> panman_core::error::Result<()> {
    unsafe {
        let mut wifi_config: esp_idf_sys::wifi_config_t = core::mem::zeroed();

        // Copy SSID (max 31 chars + null terminator)
        let ssid_bytes = ssid.as_bytes();
        let len = ssid_bytes.len().min(31);
        core::ptr::copy_nonoverlapping(
            ssid_bytes.as_ptr(),
            wifi_config.sta.ssid.as_mut_ptr(),
            len,
        );

        // Copy password (max 63 chars + null terminator)
        let pass_bytes = password.as_bytes();
        let len = pass_bytes.len().min(63);
        core::ptr::copy_nonoverlapping(
            pass_bytes.as_ptr(),
            wifi_config.sta.password.as_mut_ptr(),
            len,
        );

        check_esp(
            esp_idf_sys::esp_wifi_set_config(
                esp_idf_sys::wifi_interface_t_WIFI_IF_STA,
                &mut wifi_config,
            ),
            "esp_wifi_set_config",
        )?;
        check_esp(esp_idf_sys::esp_wifi_connect(), "esp_wifi_connect")?;
    }
    log::info!("WiFi connecting to '{}'...", ssid);
    Ok(())
}

/// Wait for WiFi to obtain an IP address, with timeout in milliseconds.
/// Returns true if connected, false if timed out.
#[cfg(target_os = "espidf")]
pub fn wait_for_connection(timeout_ms: u32) -> bool {
    let start = unsafe { esp_idf_sys::esp_timer_get_time() };
    let timeout_us = timeout_ms as i64 * 1000;

    while !is_connected() {
        if unsafe { esp_idf_sys::esp_timer_get_time() } - start > timeout_us {
            return false;
        }
        unsafe {
            esp_idf_sys::vTaskDelay(100);
        }
    }
    true
}

#[cfg(target_os = "espidf")]
unsafe extern "C" fn wifi_event_handler(
    _arg: *mut core::ffi::c_void,
    _event_base: esp_idf_sys::esp_event_base_t,
    event_id: i32,
    _event_data: *mut core::ffi::c_void,
) {
    if event_id == esp_idf_sys::wifi_event_t_WIFI_EVENT_STA_CONNECTED as i32 {
        WIFI_CONNECTED.store(true, Ordering::Relaxed);
        log::info!("WiFi: associated with AP");
    } else if event_id == esp_idf_sys::wifi_event_t_WIFI_EVENT_STA_DISCONNECTED as i32 {
        WIFI_CONNECTED.store(false, Ordering::Relaxed);
        WIFI_GOT_IP.store(false, Ordering::Relaxed);
        log::warn!("WiFi: disconnected, attempting reconnect...");
        let _ = esp_idf_sys::esp_wifi_connect();
    } else if event_id == esp_idf_sys::wifi_event_t_WIFI_EVENT_STA_START as i32 {
        log::info!("WiFi: STA started");
    }
}

#[cfg(target_os = "espidf")]
unsafe extern "C" fn ip_event_handler(
    _arg: *mut core::ffi::c_void,
    _event_base: esp_idf_sys::esp_event_base_t,
    _event_id: i32,
    event_data: *mut core::ffi::c_void,
) {
    let event = &*(event_data as *const esp_idf_sys::ip_event_got_ip_t);
    WIFI_GOT_IP.store(true, Ordering::Relaxed);
    let ip = event.ip_info.ip;
    log::info!(
        "WiFi: got IP {}.{}.{}.{}",
        ip.addr & 0xFF,
        (ip.addr >> 8) & 0xFF,
        (ip.addr >> 16) & 0xFF,
        (ip.addr >> 24) & 0xFF,
    );
}

// ---- Host stubs (for development/testing on macOS/Linux) ----

#[cfg(not(target_os = "espidf"))]
pub fn init_nvs() -> panman_core::error::Result<()> {
    log::info!("NVS: stub (host build)");
    Ok(())
}

#[cfg(not(target_os = "espidf"))]
pub fn init() -> panman_core::error::Result<()> {
    log::info!("WiFi: stub (host build)");
    Ok(())
}

#[cfg(not(target_os = "espidf"))]
pub fn connect(_ssid: &str, _password: &str) -> panman_core::error::Result<()> {
    log::info!("WiFi connect: stub (host build)");
    Ok(())
}

#[cfg(not(target_os = "espidf"))]
pub fn wait_for_connection(_timeout_ms: u32) -> bool {
    false
}
