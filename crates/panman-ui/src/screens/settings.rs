use crate::screen::{Screen, UiMessage};
use panman_core::error::Result;
use panman_core::types::{DeviceState, PropertyValue};

/// Settings screen — version, WiFi info, zman status, OTA controls.
pub struct SettingsScreen {
    firmware_version: String,
    wifi_ssid: String,
    wifi_ip: Option<String>,
    zman_url: String,
    zman_connected: bool,
    ota_available: Option<String>,
    messages: Vec<UiMessage>,
}

impl SettingsScreen {
    pub fn new(firmware_version: &str, wifi_ssid: &str, zman_url: &str) -> Self {
        Self {
            firmware_version: firmware_version.to_string(),
            wifi_ssid: wifi_ssid.to_string(),
            wifi_ip: None,
            zman_url: zman_url.to_string(),
            zman_connected: false,
            ota_available: None,
            messages: Vec::new(),
        }
    }

    pub fn set_wifi_ip(&mut self, ip: &str) {
        self.wifi_ip = Some(ip.to_string());
    }

    pub fn set_zman_connected(&mut self, connected: bool) {
        self.zman_connected = connected;
    }

    pub fn set_ota_available(&mut self, version: Option<String>) {
        self.ota_available = version;
    }

    pub fn request_ota_check(&mut self) {
        self.messages.push(UiMessage::CheckOta);
    }

    pub fn request_ota_install(&mut self) {
        self.messages.push(UiMessage::InstallOta);
    }
}

impl Screen for SettingsScreen {
    fn create(&mut self) -> Result<()> {
        log::info!(
            "Settings screen: v{}, WiFi={}, zman={}",
            self.firmware_version,
            self.wifi_ssid,
            self.zman_url
        );
        // On ESP32 with LVGL:
        // - Create labels for firmware version, WiFi SSID, IP
        // - Create connection status indicator
        // - Create "Check for Updates" button
        // - If update available, create "Install Update" button with version
        Ok(())
    }

    fn on_state_changed(&mut self, _device_id: &str, _property: &str, _value: &PropertyValue) {
        // Settings screen doesn't track device state
    }

    fn on_full_refresh(&mut self, _get_state: &dyn Fn(&str) -> Option<&DeviceState>) {
        // No-op
    }

    fn drain_messages(&mut self) -> Vec<UiMessage> {
        std::mem::take(&mut self.messages)
    }
}
