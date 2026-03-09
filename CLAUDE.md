# CLAUDE.md — Panman Project Instructions

## Project Overview

ESP32-P4 touchscreen panel firmware for home automation. Communicates with sibling `zman` project (REST API + WebSocket) to display device states and send commands. First target: CrowPanel Advanced 10.1" ESP32-P4 (1024x600 MIPI DSI).

## Build System

- **Target**: `riscv32imafc-esp-espidf` (Tier 3, requires nightly Rust + build-std)
- **ESP-IDF**: v5.4.3 (pre-installed in container)
- **Container**: `podman build -t panman-toolchain -f Containerfile .`
- **Build**: `cargo +nightly build --release --target riscv32imafc-esp-espidf -p panman-crowpanel-p4-10`
- **Container build uses**: clone from GitHub (podman VM can't mount /Volumes/minihome)
- **CI builds**: mkube job scheduling on bare metal servers (g10 network)

## Project Structure

```
Cargo.toml                         Workspace root
Containerfile                      ESP32-P4 Rust toolchain (Debian bookworm)
panman.example.toml                Example panel config

crates/
  panman-core/src/
    lib.rs, types.rs, config.rs, widget.rs, error.rs
  panman-client/src/
    lib.rs, rest.rs, ws.rs, cache.rs
  panman-ui/src/
    lib.rs, screen.rs
    screens/ (dashboard, device, settings)
    widgets/ (toggle, slider, sensor, thermostat, status)
  panman-ota/src/
    lib.rs, checker.rs, updater.rs, rollback.rs
  panman-hal/src/
    lib.rs, board.rs, boards/ (crowpanel_p4_10.rs)

boards/
  crowpanel-p4-10/
    Cargo.toml, build.rs
    src/main.rs, wifi.rs, ota_esp.rs
    sdkconfig.defaults, partitions.csv
```

## CrowPanel 10.1" Hardware

| Component | Pins/Details |
|-----------|-------------|
| Display | MIPI DSI 2-lane 1024x600, IO35-40 data, IO37-38 clock |
| Touch | GT911 I2C, SCL=IO46, SDA=IO45, INT=IO42, RST=IO40, addr 0x5D |
| Backlight | Power=IO29, PWM Enable=IO31 |
| WiFi | ESP32-C6 companion via SDIO |
| SDIO pins | CMD=IO19, CLK=IO18, D0=IO14, D1=IO15, Reset=IO32 |
| Flash | 16MB QIO |
| PSRAM | 32MB LPSRAM at 200MHz |

## Key Technical Decisions

- **Only esp-idf-sys** (git master) for P4; esp-idf-hal/svc dropped due to P4 incompatibility
- **WiFi**: Uses standard `esp_wifi_*` APIs; esp_hosted handles SDIO transport transparently
- **esp_wifi_remote version**: Must use latest (1.x+); 0.16.x has Kconfig bug causing `esp_eap_method_t` redeclaration with ESP-IDF v5.4.3
- **EAP disabled**: `CONFIG_ESP_WIFI_REMOTE_EAP_ENABLED=n` (no enterprise WiFi needed)
- **OTA**: ESP-IDF dual-partition with automatic rollback; two 7MB app slots
- **`[patch.crates-io]`**: esp-idf-sys patched to git master for P4 support
- **RISC-V c_char**: Use `c_char` (not `i8`) for HTTP client FFI casts (RISC-V `c_char` is `u8`)

## Build Quirks

- Container CMD clones from GitHub (no volume mount in podman VM on macOS)
- ESP-IDF `.git` removed in container to save space; `fatal: not a git repository` warnings are harmless
- IDF Component Manager downloads `esp_wifi_remote` and `esp_hosted` at build time
- First build in container takes ~20 min (IDF component download + full compilation); subsequent builds are faster with cached layers

## Version

Current: `0.1.0` (pre-release, initial development)

## Work Plan

### Completed
- [x] Project scaffold — workspace, all crate stubs, board binary crate
- [x] Containerfile with ESP32-P4 Rust toolchain
- [x] `.cargo/config.toml` for riscv32imafc-esp-espidf target
- [x] ESP-IDF build integration via embuild
- [x] Core types mirroring zman (DeviceId, PropertyValue, DeviceState, Event)
- [x] Config parsing (TOML)
- [x] Widget model (WidgetKind, WidgetDef)
- [x] zman REST client (list_devices, send_command)
- [x] WebSocket client stub
- [x] Device cache
- [x] Board trait + CrowPanel P4 10.1" implementation
- [x] WiFi connectivity via ESP32-C6 (esp_hosted SDIO + esp_wifi_remote)
- [x] OTA update implementation (checker, updater, rollback)
- [x] Boot sequence (WiFi -> OTA rollback -> periodic checks)
- [x] sdkconfig.defaults for CrowPanel SDIO pin mapping
- [x] Partition table (OTA dual-slot + FAT config)

### In Progress
- [ ] Container firmware build verification (esp_wifi_remote 1.x upgrade pushed, awaiting build)

### Next
- [ ] HAL: MIPI DSI display initialization
- [ ] HAL: GT911 touch driver
- [ ] HAL: PWM backlight control
- [ ] UI: LVGL integration
- [ ] UI: Dashboard screen with widget grid
- [ ] UI: Toggle/Slider/Sensor widgets
- [ ] UI: Settings screen
- [ ] Main loop: WS events -> cache -> screen -> widgets
- [ ] Push toolchain container to GHCR
