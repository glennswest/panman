use crate::screen::{Screen, UiMessage};
use crate::widgets::{create_widget, WidgetInstance};
use panman_core::config::ScreenConfig;
use panman_core::error::Result;
use panman_core::types::{DeviceState, PropertyValue};

/// Dashboard screen — grid of widget tiles showing device states.
pub struct DashboardScreen {
    config: ScreenConfig,
    widgets: Vec<WidgetInstance>,
    messages: Vec<UiMessage>,
}

impl DashboardScreen {
    pub fn new(config: ScreenConfig) -> Self {
        Self {
            config,
            widgets: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn columns(&self) -> u8 {
        self.config.columns
    }

    pub fn label(&self) -> &str {
        self.config.label.as_deref().unwrap_or(&self.config.name)
    }
}

impl Screen for DashboardScreen {
    fn create(&mut self) -> Result<()> {
        self.widgets.clear();
        for widget_def in &self.config.widgets {
            self.widgets.push(create_widget(widget_def));
        }
        log::info!(
            "Dashboard '{}': created {} widgets in {} columns",
            self.label(),
            self.widgets.len(),
            self.columns()
        );
        Ok(())
    }

    fn on_state_changed(&mut self, device_id: &str, property: &str, value: &PropertyValue) {
        for widget in &mut self.widgets {
            if widget.device_id() == Some(device_id) {
                widget.on_property_changed(property, value);
            }
        }
    }

    fn on_full_refresh(&mut self, get_state: &dyn Fn(&str) -> Option<&DeviceState>) {
        for widget in &mut self.widgets {
            let id = match widget.device_id() {
                Some(id) => id.to_string(),
                None => continue,
            };
            if let Some(state) = get_state(&id) {
                widget.update_state(state);
            }
        }
    }

    fn drain_messages(&mut self) -> Vec<UiMessage> {
        let mut msgs = std::mem::take(&mut self.messages);
        for widget in &mut self.widgets {
            msgs.extend(widget.drain_messages());
        }
        msgs
    }
}
