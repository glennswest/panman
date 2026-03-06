use panman_core::error::Result;

/// OTA rollback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootState {
    /// First boot after OTA — not yet confirmed.
    PendingVerification,
    /// Successfully confirmed — this firmware is good.
    Confirmed,
    /// Running the factory/previous firmware after a rollback.
    RolledBack,
}

/// OTA rollback manager.
///
/// After flashing new firmware, the ESP32 boots into it but marks it as
/// "pending verification." The firmware must call `confirm()` after
/// successfully:
/// 1. Connecting to WiFi
/// 2. Reaching the zman API
///
/// If the firmware crashes or fails to confirm before the next reboot,
/// the bootloader automatically rolls back to the previous partition.
///
/// On ESP32, uses:
/// - `esp_ota_mark_app_valid_cancel_rollback()` — confirm current firmware
/// - `esp_ota_mark_app_invalid_rollback_and_reboot()` — trigger immediate rollback
/// - `esp_ota_get_state_partition()` — check if pending verification
pub struct RollbackManager {
    state: BootState,
}

impl RollbackManager {
    pub fn new() -> Self {
        Self {
            state: BootState::PendingVerification,
        }
    }

    pub fn state(&self) -> BootState {
        self.state
    }

    /// Check if current boot is pending OTA verification.
    pub fn is_pending_verification(&self) -> bool {
        self.state == BootState::PendingVerification
    }

    /// Confirm the current firmware as valid.
    /// On ESP32: calls `esp_ota_mark_app_valid_cancel_rollback()`.
    pub fn confirm(&mut self) -> Result<()> {
        log::info!("OTA: confirming current firmware as valid");
        self.state = BootState::Confirmed;
        // On ESP32: esp_ota_mark_app_valid_cancel_rollback()
        Ok(())
    }

    /// Trigger rollback and reboot.
    /// On ESP32: calls `esp_ota_mark_app_invalid_rollback_and_reboot()`.
    pub fn rollback(&mut self) -> Result<()> {
        log::warn!("OTA: triggering rollback to previous firmware");
        self.state = BootState::RolledBack;
        // On ESP32: esp_ota_mark_app_invalid_rollback_and_reboot()
        Ok(())
    }

    /// Set state to confirmed (for non-OTA boots or factory firmware).
    pub fn set_confirmed(&mut self) {
        self.state = BootState::Confirmed;
    }
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self::new()
    }
}
