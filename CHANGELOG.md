# Changelog

## [Unreleased]

### 2026-03-06
- **feat:** WiFi connectivity via ESP32-C6 companion chip (esp_hosted SDIO + esp_wifi_remote)
- **feat:** OTA update implementation using ESP-IDF esp_ota_* APIs (download, flash, rollback)
- **feat:** Boot sequence with WiFi connection, OTA rollback verification, periodic OTA checks
- **feat:** sdkconfig.defaults for CrowPanel SDIO pin mapping (CMD=19, CLK=18, D0=14, D1=15, RST=32)
- **feat:** Optional WiFi password in config for development/initial provisioning

### 2026-03-05
- **feat:** Initial project scaffold — workspace, all crate stubs, board binary crate
- **feat:** Containerfile with ESP32-P4 Rust toolchain (Debian bookworm, nightly Rust, ESP-IDF v5.4)
- **feat:** `.cargo/config.toml` for riscv32imafc-esp-espidf target (build-std, ldproxy, espflash)
- **feat:** ESP-IDF build integration via embuild (sdkconfig.defaults, partition table, conditional build.rs)
- **fix:** Use only esp-idf-sys (git master) for P4; drop esp-idf-hal/svc until crates.io releases support P4
- **chore:** ESP32-P4 cross-compilation verified in container (riscv32imafc-esp-espidf, release build)
- **docs:** README, example config, CHANGELOG
