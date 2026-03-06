use panman_core::error::{PanmanError, Result};
use panman_core::types::Event;

/// WebSocket connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsState {
    Disconnected,
    Connecting,
    Connected,
}

/// WebSocket connection manager.
///
/// On ESP32, the actual WebSocket transport uses `esp-idf-svc` or a lightweight
/// WebSocket client. This struct manages connection state and event parsing.
pub struct WsConnection {
    url: String,
    state: WsState,
    reconnect_delay_ms: u32,
    max_reconnect_delay_ms: u32,
}

impl WsConnection {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            state: WsState::Disconnected,
            reconnect_delay_ms: 1000,
            max_reconnect_delay_ms: 30000,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn state(&self) -> WsState {
        self.state
    }

    pub fn set_state(&mut self, state: WsState) {
        self.state = state;
    }

    /// Parse a received WebSocket text message into an Event.
    pub fn parse_message(text: &str) -> Result<Event> {
        serde_json::from_str(text).map_err(|e| PanmanError::WebSocket(e.to_string()))
    }

    /// Get the current reconnect delay and advance it (exponential backoff).
    pub fn next_reconnect_delay(&mut self) -> u32 {
        let delay = self.reconnect_delay_ms;
        self.reconnect_delay_ms =
            (self.reconnect_delay_ms * 2).min(self.max_reconnect_delay_ms);
        delay
    }

    /// Reset reconnect delay (call after successful connection).
    pub fn reset_reconnect_delay(&mut self) {
        self.reconnect_delay_ms = 1000;
    }
}

/// Trait for WebSocket transport — implemented differently on ESP32 vs desktop.
pub trait WsTransport {
    fn connect(&mut self, url: &str) -> Result<()>;
    fn send(&mut self, text: &str) -> Result<()>;
    fn receive(&mut self) -> Result<Option<String>>;
    fn close(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
}
