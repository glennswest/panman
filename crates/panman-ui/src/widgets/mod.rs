pub mod sensor;
pub mod slider;
pub mod status;
pub mod thermostat;
pub mod toggle;

use crate::screen::UiMessage;
use panman_core::types::{DeviceState, PropertyValue};
use panman_core::widget::{WidgetDef, WidgetKind};

/// Unified widget instance that wraps the specific widget types.
pub enum WidgetInstance {
    Toggle(toggle::ToggleWidget),
    Slider(slider::SliderWidget),
    Sensor(sensor::SensorWidget),
    Thermostat(thermostat::ThermostatWidget),
    Status(status::StatusWidget),
}

impl WidgetInstance {
    pub fn device_id(&self) -> Option<&str> {
        match self {
            WidgetInstance::Toggle(w) => Some(&w.device_id),
            WidgetInstance::Slider(w) => Some(&w.device_id),
            WidgetInstance::Sensor(w) => Some(&w.device_id),
            WidgetInstance::Thermostat(w) => Some(&w.device_id),
            WidgetInstance::Status(_) => None,
        }
    }

    pub fn update_state(&mut self, state: &DeviceState) {
        match self {
            WidgetInstance::Toggle(w) => w.update_state(state),
            WidgetInstance::Slider(w) => w.update_state(state),
            WidgetInstance::Sensor(w) => w.update_state(state),
            WidgetInstance::Thermostat(w) => w.update_state(state),
            WidgetInstance::Status(_) => {}
        }
    }

    pub fn on_property_changed(&mut self, property: &str, value: &PropertyValue) {
        match self {
            WidgetInstance::Toggle(w) => w.on_property_changed(property, value),
            WidgetInstance::Slider(w) => w.on_property_changed(property, value),
            WidgetInstance::Sensor(w) => w.on_property_changed(property, value),
            WidgetInstance::Thermostat(w) => w.on_property_changed(property, value),
            WidgetInstance::Status(_) => {}
        }
    }

    pub fn drain_messages(&mut self) -> Vec<UiMessage> {
        match self {
            WidgetInstance::Toggle(w) => w.drain_messages(),
            WidgetInstance::Slider(w) => w.drain_messages(),
            WidgetInstance::Sensor(_) => Vec::new(),
            WidgetInstance::Thermostat(w) => w.drain_messages(),
            WidgetInstance::Status(_) => Vec::new(),
        }
    }
}

/// Factory function — creates the right widget type from a config definition.
pub fn create_widget(def: &WidgetDef) -> WidgetInstance {
    let label = def.label.clone().unwrap_or_default();
    let device_id = def.device_id.clone().unwrap_or_default();

    match def.kind {
        WidgetKind::Toggle => {
            WidgetInstance::Toggle(toggle::ToggleWidget::new(device_id, label))
        }
        WidgetKind::Slider => WidgetInstance::Slider(slider::SliderWidget::new(
            device_id,
            label,
            def.property.clone().unwrap_or_else(|| "brightness".into()),
            def.min.unwrap_or(0),
            def.max.unwrap_or(100),
        )),
        WidgetKind::Sensor => WidgetInstance::Sensor(sensor::SensorWidget::new(
            device_id,
            label,
            def.properties.clone(),
        )),
        WidgetKind::Thermostat => {
            WidgetInstance::Thermostat(thermostat::ThermostatWidget::new(device_id, label))
        }
        WidgetKind::Status => WidgetInstance::Status(status::StatusWidget::new(label)),
    }
}
