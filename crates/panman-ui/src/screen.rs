use panman_core::error::Result;
use panman_core::types::{DeviceState, PropertyValue};

/// Messages sent from the UI thread to the network/main thread.
#[derive(Debug, Clone)]
pub enum UiMessage {
    /// User tapped a toggle widget.
    SendCommand {
        device_id: String,
        command: String,
        params: Vec<(String, PropertyValue)>,
    },
    /// User requested navigation to a screen.
    Navigate(String),
    /// User requested OTA check.
    CheckOta,
    /// User requested OTA install.
    InstallOta,
}

/// Trait implemented by each screen (dashboard, device detail, settings).
///
/// On ESP32, `parent` would be an `*mut lv_obj_t` (LVGL object pointer).
/// This trait is defined generically so the crate compiles on any host.
/// The board binary wires it to actual LVGL objects.
pub trait Screen {
    /// Create the screen's UI elements as children of `parent`.
    fn create(&mut self) -> Result<()>;

    /// Called when a device state changes — update relevant widgets.
    fn on_state_changed(&mut self, device_id: &str, property: &str, value: &PropertyValue);

    /// Called when all device states are refreshed (e.g., after REST sync).
    fn on_full_refresh(&mut self, get_state: &dyn Fn(&str) -> Option<&DeviceState>);

    /// Collect any pending UI messages (commands, navigation, etc.).
    fn drain_messages(&mut self) -> Vec<UiMessage>;
}

/// Navigation stack manager for screens.
pub struct ScreenManager {
    screen_names: Vec<String>,
    active_index: usize,
}

impl ScreenManager {
    pub fn new(screen_names: Vec<String>) -> Self {
        Self {
            screen_names,
            active_index: 0,
        }
    }

    pub fn active_name(&self) -> &str {
        &self.screen_names[self.active_index]
    }

    pub fn active_index(&self) -> usize {
        self.active_index
    }

    pub fn screen_count(&self) -> usize {
        self.screen_names.len()
    }

    pub fn navigate_to(&mut self, name: &str) -> Option<usize> {
        if let Some(idx) = self.screen_names.iter().position(|n| n == name) {
            self.active_index = idx;
            Some(idx)
        } else {
            None
        }
    }

    pub fn navigate_next(&mut self) -> usize {
        self.active_index = (self.active_index + 1) % self.screen_names.len();
        self.active_index
    }

    pub fn navigate_prev(&mut self) -> usize {
        if self.active_index == 0 {
            self.active_index = self.screen_names.len() - 1;
        } else {
            self.active_index -= 1;
        }
        self.active_index
    }
}
