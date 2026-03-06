use crate::screen::UiMessage;
use panman_core::types::{DeviceState, PropertyValue};

/// Thermostat widget — mode selector + temperature setpoint.
pub struct ThermostatWidget {
    pub device_id: String,
    pub label: String,
    pub mode: String,
    pub setpoint: f64,
    pub current_temp: Option<f64>,
    messages: Vec<UiMessage>,
}

impl ThermostatWidget {
    pub fn new(device_id: String, label: String) -> Self {
        Self {
            device_id,
            label,
            mode: "off".into(),
            setpoint: 72.0,
            current_temp: None,
            messages: Vec::new(),
        }
    }

    pub fn update_state(&mut self, state: &DeviceState) {
        if let Some(val) = state.get("mode") {
            if let Some(m) = val.as_str() {
                self.mode = m.to_string();
            }
        }
        if let Some(val) = state.get("setpoint") {
            if let Some(v) = val.as_float() {
                self.setpoint = v;
            }
        }
        if let Some(val) = state.get("temperature") {
            self.current_temp = val.as_float();
        }
    }

    pub fn on_property_changed(&mut self, property: &str, value: &PropertyValue) {
        match property {
            "mode" => {
                if let Some(m) = value.as_str() {
                    self.mode = m.to_string();
                }
            }
            "setpoint" => {
                if let Some(v) = value.as_float() {
                    self.setpoint = v;
                }
            }
            "temperature" => {
                self.current_temp = value.as_float();
            }
            _ => {}
        }
    }

    pub fn on_mode_changed(&mut self, mode: &str) {
        self.messages.push(UiMessage::SendCommand {
            device_id: self.device_id.clone(),
            command: "set_mode".to_string(),
            params: vec![("mode".into(), PropertyValue::String(mode.into()))],
        });
        self.mode = mode.to_string();
    }

    pub fn on_setpoint_changed(&mut self, temp: f64) {
        self.messages.push(UiMessage::SendCommand {
            device_id: self.device_id.clone(),
            command: "set_setpoint".to_string(),
            params: vec![("setpoint".into(), PropertyValue::Float(temp))],
        });
        self.setpoint = temp;
    }

    pub fn drain_messages(&mut self) -> Vec<UiMessage> {
        std::mem::take(&mut self.messages)
    }
}
