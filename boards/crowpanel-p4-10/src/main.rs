mod wifi;
mod ota_esp;

use panman_core::config::PanelConfig;
use panman_ota::checker::OtaChecker;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default config used when no config file is available.
const DEFAULT_CONFIG: &str = r#"
[panel]
name = "Panel"
zman_url = "http://192.168.1.100:8081"

[wifi]
ssid = ""

[ota]
check_url = "http://192.168.1.100:8082/firmware/panman"
check_interval_secs = 3600
auto_update = false
"#;

// ---- ESP-IDF logger ----

#[cfg(target_os = "espidf")]
struct EspLogger;

#[cfg(target_os = "espidf")]
impl log::Log for EspLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        println!(
            "[{}] ({}) {}",
            record.level(),
            record.target(),
            record.args()
        );
    }
    fn flush(&self) {}
}

#[cfg(target_os = "espidf")]
static ESP_LOGGER: EspLogger = EspLogger;

fn main() {
    // Initialize logging
    #[cfg(not(target_os = "espidf"))]
    env_logger::init();

    #[cfg(target_os = "espidf")]
    {
        log::set_logger(&ESP_LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Info);
    }

    log::info!("panman v{} starting", VERSION);

    // Load config
    let config = PanelConfig::from_toml(DEFAULT_CONFIG).expect("parse default config");

    // Initialize NVS (required for WiFi credential storage)
    if let Err(e) = wifi::init_nvs() {
        log::error!("NVS init failed: {}", e);
        return;
    }

    // Initialize WiFi subsystem
    if let Err(e) = wifi::init() {
        log::error!("WiFi init failed: {}", e);
        return;
    }

    // Connect to WiFi if SSID is configured
    if !config.wifi.ssid.is_empty() {
        let password = config.wifi.password.as_deref().unwrap_or("");
        if let Err(e) = wifi::connect(&config.wifi.ssid, password) {
            log::error!("WiFi connect failed: {}", e);
        } else {
            // Wait for connection (30 second timeout)
            log::info!("Waiting for WiFi connection...");
            if wifi::wait_for_connection(30_000) {
                log::info!("WiFi connected successfully");
            } else {
                log::warn!("WiFi connection timed out (will retry in background)");
            }
        }
    } else {
        log::warn!("No WiFi SSID configured — skipping WiFi connection");
    }

    // OTA rollback handling
    if ota_esp::is_pending_verification() {
        if wifi::is_connected() {
            // WiFi is up — confirm this firmware as valid
            if let Err(e) = ota_esp::confirm_firmware() {
                log::error!("OTA confirm failed: {}", e);
            }
        } else {
            log::warn!("OTA: pending verification but WiFi not connected — will confirm later");
        }
    }

    // Check for OTA updates if configured
    if let Some(ref check_url) = config.ota.check_url {
        if wifi::is_connected() {
            log::info!("Checking for OTA updates at {}", check_url);
            check_and_apply_ota(check_url, config.ota.auto_update);
        }
    }

    log::info!("panman v{} ready", VERSION);

    // Main loop
    #[cfg(target_os = "espidf")]
    {
        let check_interval_ms = config.ota.check_interval_secs * 1000;
        let mut last_ota_check: u64 = 0;

        loop {
            unsafe {
                esp_idf_sys::vTaskDelay(1000);
            }

            // Periodic OTA check
            if let Some(ref check_url) = config.ota.check_url {
                let now = unsafe { esp_idf_sys::esp_timer_get_time() as u64 / 1000 };
                if wifi::is_connected() && now - last_ota_check > check_interval_ms as u64 {
                    last_ota_check = now;
                    check_and_apply_ota(check_url, config.ota.auto_update);
                }
            }

            // Confirm OTA if WiFi just came up after pending verification
            if ota_esp::is_pending_verification() && wifi::is_connected() {
                if let Err(e) = ota_esp::confirm_firmware() {
                    log::error!("OTA confirm failed: {}", e);
                }
            }
        }
    }

    #[cfg(not(target_os = "espidf"))]
    {
        log::info!("Running on host (not ESP32) — exiting");
    }
}

fn check_and_apply_ota(check_url: &str, auto_update: bool) {
    let checker = OtaChecker::new(check_url, VERSION);

    match ota_esp::fetch_manifest(check_url) {
        Ok(body) => match checker.check_manifest(&body) {
            Ok(Some(manifest)) => {
                log::info!(
                    "OTA: update available: v{} -> v{}",
                    VERSION,
                    manifest.version
                );
                if let Some(ref notes) = manifest.release_notes {
                    log::info!("OTA: release notes: {}", notes);
                }
                if auto_update {
                    log::info!("OTA: auto-update enabled, installing...");
                    match ota_esp::download_and_flash(&manifest.url) {
                        Ok(()) => {
                            log::info!("OTA: update installed, rebooting...");
                            #[cfg(target_os = "espidf")]
                            ota_esp::reboot();
                        }
                        Err(e) => {
                            log::error!("OTA: update failed: {}", e);
                        }
                    }
                } else {
                    log::info!("OTA: auto-update disabled, skipping install");
                }
            }
            Ok(None) => {
                log::info!("OTA: firmware is up to date (v{})", VERSION);
            }
            Err(e) => {
                log::warn!("OTA: failed to parse manifest: {}", e);
            }
        },
        Err(e) => {
            log::warn!("OTA: failed to fetch manifest: {}", e);
        }
    }
}
