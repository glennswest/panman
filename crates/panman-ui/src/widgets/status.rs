/// Status widget — connection indicator, clock, system info.
pub struct StatusWidget {
    pub label: String,
    pub wifi_connected: bool,
    pub zman_connected: bool,
    pub time_str: String,
}

impl StatusWidget {
    pub fn new(label: String) -> Self {
        Self {
            label,
            wifi_connected: false,
            zman_connected: false,
            time_str: String::new(),
        }
    }

    pub fn set_wifi_connected(&mut self, connected: bool) {
        self.wifi_connected = connected;
    }

    pub fn set_zman_connected(&mut self, connected: bool) {
        self.zman_connected = connected;
    }

    pub fn set_time(&mut self, time: &str) {
        self.time_str = time.to_string();
    }
}
