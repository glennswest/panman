use crate::widget::WidgetDef;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PanelConfig {
    pub panel: PanelSettings,
    #[serde(default)]
    pub wifi: WifiConfig,
    #[serde(default)]
    pub ota: OtaConfig,
    #[serde(default)]
    pub screens: Vec<ScreenConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PanelSettings {
    pub name: String,
    pub zman_url: String,
    #[serde(default = "default_backlight_timeout")]
    pub backlight_timeout_secs: u32,
}

fn default_backlight_timeout() -> u32 {
    120
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WifiConfig {
    pub ssid: String,
    // Password stored in NVS, not config
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtaConfig {
    #[serde(default)]
    pub check_url: Option<String>,
    #[serde(default = "default_check_interval")]
    pub check_interval_secs: u32,
    #[serde(default)]
    pub auto_update: bool,
}

impl Default for OtaConfig {
    fn default() -> Self {
        Self {
            check_url: None,
            check_interval_secs: default_check_interval(),
            auto_update: false,
        }
    }
}

fn default_check_interval() -> u32 {
    3600
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScreenConfig {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_columns")]
    pub columns: u8,
    #[serde(default)]
    pub widgets: Vec<WidgetDef>,
}

fn default_columns() -> u8 {
    3
}

impl PanelConfig {
    pub fn from_toml(s: &str) -> crate::error::Result<Self> {
        toml::from_str(s).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_config() {
        let toml_str = include_str!("../../../panman.example.toml");
        let config = PanelConfig::from_toml(toml_str).expect("parse example config");
        assert_eq!(config.panel.name, "Kitchen Panel");
        assert_eq!(config.screens.len(), 2);
        assert_eq!(config.screens[0].widgets.len(), 5);
    }
}
