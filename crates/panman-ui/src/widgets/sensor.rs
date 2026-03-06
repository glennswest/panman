use panman_core::types::{DeviceState, PropertyValue};
use std::collections::HashMap;

/// Sensor widget — read-only display for temperature, humidity, battery, etc.
pub struct SensorWidget {
    pub device_id: String,
    pub label: String,
    pub properties: Vec<String>,
    pub values: HashMap<String, PropertyValue>,
}

impl SensorWidget {
    pub fn new(device_id: String, label: String, properties: Vec<String>) -> Self {
        Self {
            device_id,
            label,
            properties,
            values: HashMap::new(),
        }
    }

    pub fn update_state(&mut self, state: &DeviceState) {
        for prop in &self.properties {
            if let Some(val) = state.get(prop) {
                self.values.insert(prop.clone(), val.clone());
            }
        }
    }

    pub fn on_property_changed(&mut self, property: &str, value: &PropertyValue) {
        if self.properties.iter().any(|p| p == property) {
            self.values.insert(property.to_string(), value.clone());
        }
    }

    /// Format a property value for display.
    pub fn format_value(property: &str, value: &PropertyValue) -> String {
        match property {
            "temperature" => {
                if let Some(v) = value.as_float() {
                    format!("{:.1}\u{00B0}F", v)
                } else {
                    "--".into()
                }
            }
            "humidity" => {
                if let Some(v) = value.as_float() {
                    format!("{:.0}%", v)
                } else {
                    "--".into()
                }
            }
            "battery" => {
                if let Some(v) = value.as_int() {
                    format!("{}%", v)
                } else {
                    "--".into()
                }
            }
            _ => match value {
                PropertyValue::Bool(v) => if *v { "Yes" } else { "No" }.into(),
                PropertyValue::Int(v) => v.to_string(),
                PropertyValue::Float(v) => format!("{:.1}", v),
                PropertyValue::String(v) => v.clone(),
            },
        }
    }
}
