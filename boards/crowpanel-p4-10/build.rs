fn main() {
    // On ESP32 builds, this would invoke the ESP-IDF build system:
    // embuild::espidf::sysenv::output();
    //
    // For native host builds (development/testing), nothing extra needed.

    #[cfg(target_os = "espidf")]
    {
        // ESP-IDF build integration handled by esp-idf-sys
        // The sdkconfig.defaults and partitions.csv are picked up automatically
    }
}
