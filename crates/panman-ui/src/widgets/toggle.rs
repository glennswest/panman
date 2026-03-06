use crate::screen::UiMessage;
use panman_core::types::{DeviceState, PropertyValue};

/// Toggle widget — on/off control for lights, switches, plugs.
pub struct ToggleWidget {
    pub device_id: String,
    pub label: String,
    pub is_on: bool,
    messages: Vec<UiMessage>,
}

impl ToggleWidget {
    pub fn new(device_id: String, label: String) -> Self {
        Self {
            device_id,
            label,
            is_on: false,
            messages: Vec::new(),
        }
    }

    pub fn update_state(&mut self, state: &DeviceState) {
        if let Some(val) = state.get("on") {
            if let Some(on) = val.as_bool() {
                self.is_on = on;
            }
        }
    }

    pub fn on_property_changed(&mut self, property: &str, value: &PropertyValue) {
        if property == "on" {
            if let Some(on) = value.as_bool() {
                self.is_on = on;
            }
        }
    }

    /// Called when user taps the toggle. Sends the opposite state.
    pub fn on_tap(&mut self) {
        let new_state = !self.is_on;
        self.messages.push(UiMessage::SendCommand {
            device_id: self.device_id.clone(),
            command: if new_state { "on" } else { "off" }.to_string(),
            params: Vec::new(),
        });
        // Optimistic update
        self.is_on = new_state;
    }

    pub fn drain_messages(&mut self) -> Vec<UiMessage> {
        std::mem::take(&mut self.messages)
    }
}
