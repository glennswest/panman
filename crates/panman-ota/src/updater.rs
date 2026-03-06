use crate::checker::FirmwareManifest;
use panman_core::error::{PanmanError, Result};

/// OTA update progress.
#[derive(Debug, Clone, Copy)]
pub struct OtaProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub phase: OtaPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaPhase {
    Downloading,
    Verifying,
    Flashing,
    Complete,
    Failed,
}

/// OTA updater — downloads firmware, verifies checksum, flashes to inactive partition.
///
/// On ESP32, uses `esp_ota_*` APIs from `esp-idf-svc`:
/// - `esp_ota_begin()` — open the inactive app partition for writing
/// - `esp_ota_write()` — write firmware chunks as they download
/// - `esp_ota_end()` — finalize and verify
/// - `esp_ota_set_boot_partition()` — set the new partition as boot target
///
/// The actual HTTP download and flash are delegated to platform-specific code.
/// This struct manages the state machine and validation logic.
pub struct OtaUpdater {
    manifest: FirmwareManifest,
    progress: OtaProgress,
}

impl OtaUpdater {
    pub fn new(manifest: FirmwareManifest) -> Self {
        Self {
            progress: OtaProgress {
                bytes_downloaded: 0,
                total_bytes: manifest.size,
                phase: OtaPhase::Downloading,
            },
            manifest,
        }
    }

    pub fn manifest(&self) -> &FirmwareManifest {
        &self.manifest
    }

    pub fn progress(&self) -> &OtaProgress {
        &self.progress
    }

    pub fn firmware_url(&self) -> &str {
        &self.manifest.url
    }

    pub fn expected_sha256(&self) -> &str {
        &self.manifest.sha256
    }

    /// Update download progress.
    pub fn on_bytes_received(&mut self, bytes: u64) {
        self.progress.bytes_downloaded += bytes;
    }

    /// Transition to verification phase.
    pub fn begin_verify(&mut self) {
        self.progress.phase = OtaPhase::Verifying;
    }

    /// Verify the SHA256 checksum matches.
    pub fn verify_checksum(&self, computed_sha256: &str) -> Result<()> {
        if computed_sha256 == self.manifest.sha256 {
            Ok(())
        } else {
            Err(PanmanError::Ota(format!(
                "checksum mismatch: expected {}, got {}",
                self.manifest.sha256, computed_sha256
            )))
        }
    }

    /// Transition to flashing phase.
    pub fn begin_flash(&mut self) {
        self.progress.phase = OtaPhase::Flashing;
    }

    /// Mark update as complete.
    pub fn complete(&mut self) {
        self.progress.phase = OtaPhase::Complete;
    }

    /// Mark update as failed.
    pub fn fail(&mut self) {
        self.progress.phase = OtaPhase::Failed;
    }

    /// Get download progress as percentage (0-100), if total size is known.
    pub fn percent_complete(&self) -> Option<u8> {
        self.progress.total_bytes.map(|total| {
            if total == 0 {
                100
            } else {
                ((self.progress.bytes_downloaded * 100) / total).min(100) as u8
            }
        })
    }
}
