# ESP32-P4 Rust Development Toolchain
#
# Builds panman firmware for CrowPanel Advanced 10.1" ESP32-P4
# Target: riscv32imafc-esp-espidf (RISC-V with hardware FPU + atomics)
#
# Build (auto-detects host arch — works on both ARM64 and x86_64):
#   podman build -t panman-toolchain -f Containerfile .
#
# Build firmware:
#   podman run --rm -v $(pwd):/work panman-toolchain
#
# Interactive shell:
#   podman run --rm -it -v $(pwd):/work panman-toolchain bash

FROM docker.io/library/debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

# System dependencies for ESP-IDF + Rust + bindgen
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    ninja-build \
    python3 \
    python3-venv \
    python-is-python3 \
    python3-pip \
    git \
    curl \
    wget \
    pkg-config \
    libssl-dev \
    llvm-dev \
    libclang-dev \
    clang \
    libudev-dev \
    libusb-1.0-0-dev \
    flex \
    bison \
    gperf \
    ccache \
    dfu-util \
    ca-certificates \
    unzip \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

# Install Rust (nightly for build-std, required by Tier 3 target)
ENV RUSTUP_HOME=/opt/rust/rustup
ENV CARGO_HOME=/opt/rust/cargo
ENV PATH="/opt/rust/cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
    -y \
    --default-toolchain nightly \
    --profile minimal \
    && rustup component add rust-src

# Install ESP32 Rust tools (ldproxy for linking, espflash for flashing)
RUN cargo install ldproxy espflash --locked \
    && rm -rf ${CARGO_HOME}/registry ${CARGO_HOME}/git

# Install ESP-IDF v5.4 (pre-installed so first build is fast)
ENV IDF_PATH=/opt/esp-idf
ENV IDF_TOOLS_PATH=/opt/espressif

RUN git clone --branch v5.4 --depth 1 --shallow-submodules --recursive \
    https://github.com/espressif/esp-idf.git ${IDF_PATH} \
    && ${IDF_PATH}/install.sh esp32p4 \
    && rm -rf ${IDF_PATH}/.git

# Tell esp-idf-sys native builder to use pre-installed ESP-IDF
ENV ESP_IDF_TOOLS_INSTALL_DIR=global
ENV ESP_IDF_GLOB_BASE="${IDF_PATH}"

WORKDIR /work

# Default: clone repo from GitHub and build the CrowPanel firmware
CMD bash -c "git clone https://github.com/glennswest/panman.git /work/panman && \
    cd /work/panman && \
    cargo +nightly build --release \
      --target riscv32imafc-esp-espidf \
      -p panman-crowpanel-p4-10"
