use panman_core::types::*;
use std::collections::HashMap;

/// In-memory device state cache.
///
/// Bootstrapped from REST API response, then kept current via WebSocket events.
pub struct DeviceCache {
    devices: HashMap<DeviceId, CachedDevice>,
}

pub struct CachedDevice {
    pub info: Option<DeviceInfo>,
    pub state: DeviceState,
}

impl DeviceCache {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Bootstrap cache from REST API response.
    pub fn load_devices(&mut self, entries: Vec<DeviceEntry>) {
        self.devices.clear();
        for entry in entries {
            let id = entry
                .info
                .as_ref()
                .map(|i| i.id.clone())
                .unwrap_or_default();
            if id.is_empty() {
                continue;
            }
            self.devices.insert(
                id,
                CachedDevice {
                    info: entry.info,
                    state: entry.state.unwrap_or_default(),
                },
            );
        }
    }

    /// Apply a WebSocket event to the cache. Returns the affected device ID
    /// if the event caused a state change.
    pub fn apply_event(&mut self, event: &Event) -> Option<DeviceId> {
        match event {
            Event::StateChanged {
                device_id,
                property,
                value,
            } => {
                if let Some(device) = self.devices.get_mut(device_id) {
                    device.state.set(property.clone(), value.clone());
                    Some(device_id.clone())
                } else {
                    // Device not in cache — create a minimal entry
                    let mut state = DeviceState::default();
                    state.set(property.clone(), value.clone());
                    self.devices.insert(
                        device_id.clone(),
                        CachedDevice { info: None, state },
                    );
                    Some(device_id.clone())
                }
            }
            Event::DeviceAdded { device_id, info } => {
                self.devices.insert(
                    device_id.clone(),
                    CachedDevice {
                        info: Some(info.clone()),
                        state: DeviceState::default(),
                    },
                );
                Some(device_id.clone())
            }
            Event::DeviceRemoved { device_id } => {
                self.devices.remove(device_id);
                Some(device_id.clone())
            }
            Event::DeviceUpdated { device_id, info } => {
                if let Some(device) = self.devices.get_mut(device_id) {
                    device.info = Some(info.clone());
                } else {
                    self.devices.insert(
                        device_id.clone(),
                        CachedDevice {
                            info: Some(info.clone()),
                            state: DeviceState::default(),
                        },
                    );
                }
                Some(device_id.clone())
            }
        }
    }

    pub fn get(&self, device_id: &str) -> Option<&CachedDevice> {
        self.devices.get(device_id)
    }

    pub fn get_state(&self, device_id: &str) -> Option<&DeviceState> {
        self.devices.get(device_id).map(|d| &d.state)
    }

    pub fn get_info(&self, device_id: &str) -> Option<&DeviceInfo> {
        self.devices.get(device_id).and_then(|d| d.info.as_ref())
    }

    pub fn device_ids(&self) -> impl Iterator<Item = &DeviceId> {
        self.devices.keys()
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

impl Default for DeviceCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_query() {
        let mut cache = DeviceCache::new();
        let entries = vec![DeviceEntry {
            info: Some(DeviceInfo {
                id: "zwave.node_5".into(),
                name: "Kitchen Light".into(),
                category: DeviceCategory::Light,
                manufacturer: None,
                model: None,
                location: None,
            }),
            state: Some({
                let mut s = DeviceState::default();
                s.set("on".into(), PropertyValue::Bool(true));
                s
            }),
        }];
        cache.load_devices(entries);
        assert_eq!(cache.len(), 1);
        let state = cache.get_state("zwave.node_5").unwrap();
        assert_eq!(state.get("on"), Some(&PropertyValue::Bool(true)));
    }

    #[test]
    fn apply_state_changed() {
        let mut cache = DeviceCache::new();
        let entries = vec![DeviceEntry {
            info: Some(DeviceInfo {
                id: "zwave.node_5".into(),
                name: "Kitchen Light".into(),
                category: DeviceCategory::Light,
                manufacturer: None,
                model: None,
                location: None,
            }),
            state: Some(DeviceState::default()),
        }];
        cache.load_devices(entries);

        let event = Event::StateChanged {
            device_id: "zwave.node_5".into(),
            property: "on".into(),
            value: PropertyValue::Bool(false),
        };
        let affected = cache.apply_event(&event);
        assert_eq!(affected, Some("zwave.node_5".into()));
        let state = cache.get_state("zwave.node_5").unwrap();
        assert_eq!(state.get("on"), Some(&PropertyValue::Bool(false)));
    }
}
