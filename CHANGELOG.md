# Changelog

## [Unreleased]

### 2026-03-06
- **feat:** WiFi connectivity via ESP32-C6 companion chip (esp_hosted SDIO + esp_wifi_remote)
- **feat:** OTA update implementation using ESP-IDF esp_ota_* APIs (download, flash, rollback)
- **feat:** Boot sequence with WiFi connection, OTA rollback verification, periodic OTA checks
- **feat:** sdkconfig.defaults for CrowPanel SDIO pin mapping (CMD=19, CLK=18, D0=14, D1=15, RST=32)
- **feat:** Optional WiFi password in config for development/initial provisioning
- **fix:** Use `c_char` for HTTP client FFI casts (RISC-V `c_char` is `u8`, not `i8`)
- **fix:** Upgrade ESP-IDF to v5.4.3 — resolves `esp_eap_method_t` undefined type (typedef added in v5.4.3, not present in v5.4.1)
- **fix:** Upgrade `esp_wifi_remote` from `~0.16` to `*` (latest 1.x) — resolves `esp_eap_method_t` redeclaration conflict with ESP-IDF v5.4.3 (Kconfig version expansion bug in 0.16.x)

### 2026-03-05
- **feat:** Initial project scaffold — workspace, all crate stubs, board binary crate
- **feat:** Containerfile with ESP32-P4 Rust toolchain (Debian bookworm, nightly Rust, ESP-IDF v5.4)
- **feat:** `.cargo/config.toml` for riscv32imafc-esp-espidf target (build-std, ldproxy, espflash)
- **feat:** ESP-IDF build integration via embuild (sdkconfig.defaults, partition table, conditional build.rs)
- **fix:** Use only esp-idf-sys (git master) for P4; drop esp-idf-hal/svc until crates.io releases support P4
- **chore:** ESP32-P4 cross-compilation verified in container (riscv32imafc-esp-espidf, release build)
- **docs:** README, example config, CHANGELOG
