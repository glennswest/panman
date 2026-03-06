use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type DeviceId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCategory {
    Light,
    Switch,
    Dimmer,
    Sensor,
    Thermostat,
    Lock,
    Cover,
    Fan,
    Plug,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl PropertyValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            PropertyValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(v) => Some(*v),
            PropertyValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            PropertyValue::String(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub category: DeviceCategory,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceState {
    #[serde(flatten)]
    pub properties: HashMap<String, PropertyValue>,
}

impl DeviceState {
    pub fn get(&self, property: &str) -> Option<&PropertyValue> {
        self.properties.get(property)
    }

    pub fn set(&mut self, property: String, value: PropertyValue) {
        self.properties.insert(property, value);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEntry {
    pub info: Option<DeviceInfo>,
    pub state: Option<DeviceState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    #[serde(default)]
    pub params: HashMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub status: String,
    #[serde(default)]
    pub command_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPoint {
    pub timestamp: u64,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    StateChanged {
        device_id: DeviceId,
        property: String,
        value: PropertyValue,
    },
    DeviceAdded {
        device_id: DeviceId,
        info: DeviceInfo,
    },
    DeviceRemoved {
        device_id: DeviceId,
    },
    DeviceUpdated {
        device_id: DeviceId,
        info: DeviceInfo,
    },
}
