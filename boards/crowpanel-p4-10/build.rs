fn main() {
    // embuild handles ESP-IDF build system integration:
    // - Downloads ESP-IDF v5.4 if not pre-installed
    // - Installs RISC-V GCC toolchain
    // - Runs CMake/Ninja to build ESP-IDF components
    // - Generates Rust FFI bindings via bindgen
    //
    // CARGO_CFG_TARGET_OS is set by Cargo to the *target* OS (not the host),
    // so this correctly gates the ESP-IDF build on cross-compilation.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("espidf") {
        embuild::espidf::sysenv::output();
    }
}
