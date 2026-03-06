use crate::screen::{Screen, UiMessage};
use panman_core::error::Result;
use panman_core::types::{DeviceInfo, DeviceState, PropertyValue};

/// Single device detail screen — shows all properties and controls.
pub struct DeviceScreen {
    device_id: String,
    info: Option<DeviceInfo>,
    state: DeviceState,
    messages: Vec<UiMessage>,
}

impl DeviceScreen {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            info: None,
            state: DeviceState::default(),
            messages: Vec::new(),
        }
    }

    pub fn set_info(&mut self, info: DeviceInfo) {
        self.info = Some(info);
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn send_command(&mut self, command: &str, params: Vec<(String, PropertyValue)>) {
        self.messages.push(UiMessage::SendCommand {
            device_id: self.device_id.clone(),
            command: command.to_string(),
            params,
        });
    }
}

impl Screen for DeviceScreen {
    fn create(&mut self) -> Result<()> {
        log::info!("Device screen: {}", self.device_id);
        Ok(())
    }

    fn on_state_changed(&mut self, device_id: &str, property: &str, value: &PropertyValue) {
        if device_id == self.device_id {
            self.state.set(property.to_string(), value.clone());
        }
    }

    fn on_full_refresh(&mut self, get_state: &dyn Fn(&str) -> Option<&DeviceState>) {
        if let Some(state) = get_state(&self.device_id) {
            self.state = state.clone();
        }
    }

    fn drain_messages(&mut self) -> Vec<UiMessage> {
        std::mem::take(&mut self.messages)
    }
}
