use crate::screen::UiMessage;
use panman_core::types::{DeviceState, PropertyValue};

/// Slider widget — continuous control for dimmers, fan speed, cover position.
pub struct SliderWidget {
    pub device_id: String,
    pub label: String,
    pub property: String,
    pub min: i64,
    pub max: i64,
    pub current: i64,
    messages: Vec<UiMessage>,
}

impl SliderWidget {
    pub fn new(device_id: String, label: String, property: String, min: i64, max: i64) -> Self {
        Self {
            device_id,
            label,
            property,
            min,
            max,
            current: min,
            messages: Vec::new(),
        }
    }

    pub fn update_state(&mut self, state: &DeviceState) {
        if let Some(val) = state.get(&self.property) {
            if let Some(v) = val.as_int() {
                self.current = v.clamp(self.min, self.max);
            } else if let Some(v) = val.as_float() {
                self.current = (v as i64).clamp(self.min, self.max);
            }
        }
    }

    pub fn on_property_changed(&mut self, property: &str, value: &PropertyValue) {
        if property == self.property {
            if let Some(v) = value.as_int() {
                self.current = v.clamp(self.min, self.max);
            } else if let Some(v) = value.as_float() {
                self.current = (v as i64).clamp(self.min, self.max);
            }
        }
    }

    /// Called when user changes the slider value.
    pub fn on_value_changed(&mut self, value: i64) {
        let clamped = value.clamp(self.min, self.max);
        self.messages.push(UiMessage::SendCommand {
            device_id: self.device_id.clone(),
            command: "set".to_string(),
            params: vec![(self.property.clone(), PropertyValue::Int(clamped))],
        });
        self.current = clamped;
    }

    pub fn drain_messages(&mut self) -> Vec<UiMessage> {
        std::mem::take(&mut self.messages)
    }
}
