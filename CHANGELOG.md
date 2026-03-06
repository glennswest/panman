# Changelog

## [Unreleased]

### 2026-03-05
- **feat:** Initial project scaffold — workspace, all crate stubs, board binary crate
- **feat:** Containerfile with ESP32-P4 Rust toolchain (Debian bookworm, nightly Rust, ESP-IDF v5.4)
- **feat:** `.cargo/config.toml` for riscv32imafc-esp-espidf target (build-std, ldproxy, espflash)
- **feat:** ESP-IDF build integration via embuild (sdkconfig.defaults, partition table, conditional build.rs)
- **fix:** Patch esp-idf-hal/svc/sys to git master for ESP32-P4 support (crates.io 0.45.2 predates P4 fix PRs)
- **docs:** README, example config, CHANGELOG
