use panman_core::error::Result;
use panman_core::types::*;
use std::collections::HashMap;

/// HTTP client for the zman REST API.
///
/// On ESP32, this will use `esp-idf-svc::http::client`. For now, the interface
/// is defined with method signatures that the HAL-specific HTTP implementation
/// will fulfill via a trait.
pub struct ZmanClient {
    base_url: String,
}

impl ZmanClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the URL for listing all devices.
    pub fn devices_url(&self) -> String {
        format!("{}/api/v1/devices", self.base_url)
    }

    /// Build the URL for a single device.
    pub fn device_url(&self, id: &str) -> String {
        format!("{}/api/v1/devices/{}", self.base_url, id)
    }

    /// Build the URL for sending a command to a device.
    pub fn command_url(&self, id: &str) -> String {
        format!("{}/api/v1/devices/{}/command", self.base_url, id)
    }

    /// Build the URL for device property history.
    pub fn history_url(&self, id: &str, property: &str, range: &str) -> String {
        format!(
            "{}/api/v1/history/{}/{}?range={}",
            self.base_url, id, property, range
        )
    }

    /// Build the WebSocket URL.
    pub fn ws_url(&self) -> String {
        let ws_base = self
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        format!("{}/ws", ws_base)
    }

    /// Parse a device list response body.
    pub fn parse_devices(body: &[u8]) -> Result<Vec<DeviceEntry>> {
        serde_json::from_slice(body).map_err(Into::into)
    }

    /// Parse a single device response body.
    pub fn parse_device(body: &[u8]) -> Result<DeviceEntry> {
        serde_json::from_slice(body).map_err(Into::into)
    }

    /// Build a command request body.
    pub fn build_command(command: &str, params: HashMap<String, PropertyValue>) -> Result<Vec<u8>> {
        let req = CommandRequest {
            command: command.to_string(),
            params,
        };
        serde_json::to_vec(&req).map_err(Into::into)
    }

    /// Parse a command response body.
    pub fn parse_command_response(body: &[u8]) -> Result<CommandResponse> {
        serde_json::from_slice(body).map_err(Into::into)
    }

    /// Parse history response body.
    pub fn parse_history(body: &[u8]) -> Result<Vec<HistoryPoint>> {
        serde_json::from_slice(body).map_err(Into::into)
    }
}

/// Trait for HTTP transport — implemented differently on ESP32 vs desktop.
pub trait HttpTransport {
    fn get(&mut self, url: &str) -> Result<Vec<u8>>;
    fn post(&mut self, url: &str, body: &[u8]) -> Result<Vec<u8>>;
}

/// High-level client that combines URL building with HTTP transport.
pub struct ConnectedClient<T: HttpTransport> {
    client: ZmanClient,
    transport: T,
}

impl<T: HttpTransport> ConnectedClient<T> {
    pub fn new(base_url: &str, transport: T) -> Self {
        Self {
            client: ZmanClient::new(base_url),
            transport,
        }
    }

    pub fn list_devices(&mut self) -> Result<Vec<DeviceEntry>> {
        let url = self.client.devices_url();
        let body = self.transport.get(&url)?;
        ZmanClient::parse_devices(&body)
    }

    pub fn get_device(&mut self, id: &str) -> Result<DeviceEntry> {
        let url = self.client.device_url(id);
        let body = self.transport.get(&url)?;
        ZmanClient::parse_device(&body)
    }

    pub fn send_command(
        &mut self,
        device_id: &str,
        command: &str,
        params: HashMap<String, PropertyValue>,
    ) -> Result<CommandResponse> {
        let url = self.client.command_url(device_id);
        let body = ZmanClient::build_command(command, params)?;
        let resp = self.transport.post(&url, &body)?;
        ZmanClient::parse_command_response(&resp)
    }

    pub fn ws_url(&self) -> String {
        self.client.ws_url()
    }
}
