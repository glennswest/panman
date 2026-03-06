# panman

ESP32 touchscreen panel firmware for home automation. Communicates with [zman](../zman) to display device states and send commands.

## Supported Panels

| Panel | SoC | Display | Touch | Status |
|-------|-----|---------|-------|--------|
| CrowPanel Advanced 10.1" | ESP32-P4 | 1024x600 MIPI DSI | GT911 I2C | In Development |

## Architecture

```
panman-core      Shared types, config, widget model
panman-client    zman REST + WebSocket client
panman-ui        LVGL-based UI: widgets, screens, navigation
panman-ota       OTA update subsystem (dual-partition, rollback)
panman-hal       Hardware abstraction (Board trait + implementations)
```

Each panel has a binary crate under `boards/` with board-specific ESP-IDF config.

## Build

Requires [esp-idf-sys](https://github.com/esp-rs/esp-idf-sys) toolchain.

```bash
# CrowPanel 10.1" ESP32-P4
cd boards/crowpanel-p4-10
cargo build --release --target riscv32imafc-esp-espidf

# Flash + monitor
espflash flash target/riscv32imafc-esp-espidf/release/panman-crowpanel-p4-10 --monitor
```

## Configuration

Copy `panman.example.toml` and customize for your setup. WiFi password is stored in NVS (non-volatile storage), not in the config file.

## Communication

- **REST**: Initial device sync via `GET /api/v1/devices`, commands via `POST /api/v1/devices/{id}/command`
- **WebSocket**: Real-time state updates via `/ws` — `StateChanged` events keep the UI live

## OTA Updates

Dual-partition OTA with automatic rollback. The panel polls a configurable URL for firmware manifests. New firmware must successfully connect to WiFi and reach zman before confirming itself — otherwise it rolls back on reboot.
