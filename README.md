# panman

ESP32-P4 touchscreen panel firmware for home automation. Communicates with [zman](https://github.com/glennswest/zman) via REST + WebSocket to display device states and send commands.

## Supported Panels

| Panel | SoC | Display | Touch | WiFi | Status |
|-------|-----|---------|-------|------|--------|
| CrowPanel Advanced 10.1" | ESP32-P4 | 1024x600 MIPI DSI | GT911 I2C | ESP32-C6 via SDIO | In Development |

## Architecture

```
panman/
  crates/
    panman-core/       Shared types, config (TOML), widget model, error types
    panman-client/     zman REST client + WebSocket (real-time state updates)
    panman-ui/         UI: screens, widgets (toggle, slider, sensor, thermostat, status)
    panman-ota/        OTA updates (version check, download, flash, rollback)
    panman-hal/        Hardware abstraction (Board trait, per-board implementations)
  boards/
    crowpanel-p4-10/   Binary crate for CrowPanel 10.1" ESP32-P4
```

Platform-agnostic library crates with ESP32-specific implementations behind `cfg(target_os = "espidf")` gates. Adding a new panel = new `Board` impl + new binary crate.

## Build

### Container Build (recommended)

The toolchain container has everything pre-installed (Rust nightly, ESP-IDF v5.4.3, RISC-V tools).

```bash
# Build the toolchain container
podman build -t panman-toolchain -f Containerfile .

# Build firmware (clones from GitHub inside container)
podman run --rm panman-toolchain

# Interactive development shell
podman run --rm -it -v $(pwd):/work panman-toolchain bash
cd /work && cargo +nightly build --release --target riscv32imafc-esp-espidf -p panman-crowpanel-p4-10
```

### Local Build

Requires nightly Rust with `rust-src`, ESP-IDF v5.4.3, and RISC-V toolchain.

```bash
cargo +nightly build --release --target riscv32imafc-esp-espidf -p panman-crowpanel-p4-10
```

### Flash + Monitor

```bash
espflash flash target/riscv32imafc-esp-espidf/release/panman-crowpanel-p4-10 --monitor
```

## CrowPanel Hardware

| Component | Details |
|-----------|---------|
| Display | MIPI DSI 2-lane, 1024x600, pins IO35-40 data, IO37-38 clock |
| Touch | GT911 I2C, SCL=IO46, SDA=IO45, INT=IO42, RST=IO40, addr 0x5D |
| Backlight | Power=IO29, PWM Enable=IO31 |
| WiFi | ESP32-C6 companion via SDIO (CMD=IO19, CLK=IO18, D0=IO14, D1=IO15, Reset=IO32) |
| Flash | 16MB QIO |
| PSRAM | 32MB LPSRAM at 200MHz |

## Configuration

Copy `panman.example.toml` and customize. WiFi password should be stored in NVS (non-volatile storage) for production; the config file password field is for initial provisioning only.

```toml
[panel]
name = "Kitchen Panel"
zman_url = "http://192.168.1.100:8081"

[wifi]
ssid = "HomeNetwork"

[ota]
check_url = "http://192.168.1.100:8082/firmware/panman"
check_interval_secs = 3600
auto_update = false

[[screens]]
name = "dashboard"
columns = 3

[[screens.widgets]]
kind = "toggle"
device_id = "zwave.node_5"
label = "Kitchen Light"
```

## Communication

- **REST**: Device sync via `GET /api/v1/devices`, commands via `POST /api/v1/devices/{id}/command`
- **WebSocket**: Real-time `StateChanged` events via `/ws`

## OTA Updates

ESP-IDF dual-partition OTA with automatic rollback:
- Two 7MB app slots + NVS + FAT config partition (16MB flash)
- Panel polls configurable URL for `{version, url, sha256}` manifest
- New firmware must connect WiFi + reach zman API before confirming (else rollback on reboot)

## Key Dependencies

| Crate/Component | Purpose |
|-----------------|---------|
| `esp-idf-sys` (git master) | ESP-IDF FFI bindings for P4 (Tier 3 target) |
| `espressif/esp_wifi_remote` | Routes `esp_wifi_*` API to companion chip |
| `espressif/esp_hosted` | SDIO transport to ESP32-C6 companion |
| `embuild` | ESP-IDF CMake build integration |

## License

MIT
