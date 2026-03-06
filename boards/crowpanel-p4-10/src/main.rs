use panman_client::cache::DeviceCache;
use panman_client::rest::ZmanClient;
use panman_client::ws::WsConnection;
use panman_core::config::PanelConfig;
use panman_hal::board::Board;
use panman_hal::boards::crowpanel_p4_10::CrowPanelP4_10;
use panman_ota::checker::OtaChecker;
use panman_ota::rollback::RollbackManager;
use panman_ui::screen::{Screen, ScreenManager};
use panman_ui::screens::dashboard::DashboardScreen;
use panman_ui::screens::settings::SettingsScreen;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default config used when no config file is available.
const DEFAULT_CONFIG: &str = r#"
[panel]
name = "Panel"
zman_url = "http://192.168.1.100:8081"

[wifi]
ssid = ""
"#;

fn main() {
    // Initialize logging
    #[cfg(not(target_os = "espidf"))]
    env_logger::init();

    // On ESP32: esp_idf_svc::log::EspLogger::initialize_default()

    log::info!("panman v{} starting on {}", VERSION, CrowPanelP4_10::name());

    // Load config (from NVS on ESP32, or default for development)
    let config = PanelConfig::from_toml(DEFAULT_CONFIG).expect("parse default config");

    // Initialize board hardware
    let peripherals = CrowPanelP4_10::init().expect("board init");
    log::info!(
        "Display: {}x{}",
        peripherals.width,
        peripherals.height
    );

    // Set backlight on
    CrowPanelP4_10::set_backlight(100).ok();

    // Initialize OTA rollback manager
    let mut rollback = RollbackManager::new();

    // Create zman client
    let zman = ZmanClient::new(&config.panel.zman_url);
    log::info!("zman API: {}", zman.base_url());

    // Create WebSocket connection
    let ws = WsConnection::new(&zman.ws_url());
    log::info!("WebSocket: {}", ws.url());

    // Create device cache
    let _cache = DeviceCache::new();

    // Create OTA checker if configured
    let _ota_checker = config.ota.check_url.as_ref().map(|url| {
        OtaChecker::new(url, VERSION)
    });

    // Create screens
    let mut screens: Vec<Box<dyn Screen>> = Vec::new();
    let mut screen_names: Vec<String> = Vec::new();

    for screen_cfg in &config.screens {
        match screen_cfg.name.as_str() {
            "settings" => {
                let settings = SettingsScreen::new(VERSION, &config.wifi.ssid, &config.panel.zman_url);
                screens.push(Box::new(settings));
                screen_names.push("settings".into());
            }
            _ => {
                // Default to dashboard screen
                let dashboard = DashboardScreen::new(screen_cfg.clone());
                screens.push(Box::new(dashboard));
                screen_names.push(screen_cfg.name.clone());
            }
        }
    }

    // If no screens defined, create a default dashboard
    if screens.is_empty() {
        let settings = SettingsScreen::new(VERSION, &config.wifi.ssid, &config.panel.zman_url);
        screens.push(Box::new(settings));
        screen_names.push("settings".into());
    }

    let screen_mgr = ScreenManager::new(screen_names);

    // Create all screens
    for screen in &mut screens {
        screen.create().expect("screen create");
    }

    log::info!(
        "UI ready: {} screens, active='{}'",
        screen_mgr.screen_count(),
        screen_mgr.active_name()
    );

    // On ESP32, the main loop would:
    // 1. Core 0: LVGL timer handler (lv_timer_handler) every 5ms
    //    - Read touch input -> feed to LVGL
    //    - LVGL renders dirty areas to display buffer -> flush callback
    // 2. Core 1 (async tasks):
    //    - WiFi connection management
    //    - REST sync: GET /api/v1/devices -> cache.load_devices()
    //    - WebSocket: receive events -> cache.apply_event() -> screen.on_state_changed()
    //    - OTA: periodic check -> prompt user or auto-update
    //    - Backlight timeout: dim after inactivity, wake on touch
    // 3. Cross-core communication via std::sync::mpsc:
    //    - UiMessage::SendCommand -> REST POST
    //    - UiMessage::Navigate -> screen_mgr.navigate_to()
    //    - UiMessage::CheckOta -> ota_checker.check()
    //    - UiMessage::InstallOta -> ota_updater.start()

    // Confirm OTA if this is a fresh update (after WiFi + zman verified)
    if rollback.is_pending_verification() {
        // In real firmware: only confirm after WiFi + zman connection succeeds
        rollback.confirm().ok();
        log::info!("OTA firmware confirmed");
    }

    log::info!("panman ready — waiting for ESP32 target build to run");

    // On native host, just exit. On ESP32, this would be an infinite loop.
    #[cfg(not(target_os = "espidf"))]
    {
        log::info!("Running on host (not ESP32) — exiting");
    }
}
