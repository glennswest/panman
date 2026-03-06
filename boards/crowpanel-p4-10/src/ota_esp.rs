//! OTA update implementation using ESP-IDF APIs.
//!
//! On ESP32, uses:
//! - `esp_ota_begin/write/end` for firmware flashing
//! - `esp_ota_set_boot_partition` to activate new firmware
//! - `esp_ota_mark_app_valid_cancel_rollback` to confirm after successful boot
//! - `esp_ota_get_state_partition` to check rollback state
//! - `esp_http_client` for downloading firmware and checking updates

use panman_core::error::PanmanError;

// ---- ESP-IDF target implementation ----

#[cfg(target_os = "espidf")]
fn check_esp_ota(result: esp_idf_sys::esp_err_t, context: &str) -> panman_core::error::Result<()> {
    if result == esp_idf_sys::ESP_OK {
        Ok(())
    } else {
        Err(PanmanError::Ota(format!("{}: error 0x{:x}", context, result)))
    }
}

/// Check if the current boot is pending OTA verification.
/// Returns true if we're running newly-flashed firmware that hasn't been confirmed yet.
#[cfg(target_os = "espidf")]
pub fn is_pending_verification() -> bool {
    unsafe {
        let partition = esp_idf_sys::esp_ota_get_running_partition();
        if partition.is_null() {
            return false;
        }
        let mut state: esp_idf_sys::esp_ota_img_states_t = 0;
        let ret = esp_idf_sys::esp_ota_get_state_partition(partition, &mut state);
        if ret != esp_idf_sys::ESP_OK {
            return false;
        }
        state == esp_idf_sys::esp_ota_img_states_t_ESP_OTA_IMG_PENDING_VERIFY
    }
}

/// Confirm the current firmware as valid (cancel rollback).
/// Call this after WiFi + zman connection is verified.
#[cfg(target_os = "espidf")]
pub fn confirm_firmware() -> panman_core::error::Result<()> {
    unsafe {
        check_esp_ota(
            esp_idf_sys::esp_ota_mark_app_valid_cancel_rollback(),
            "esp_ota_mark_app_valid_cancel_rollback",
        )?;
    }
    log::info!("OTA: firmware confirmed as valid");
    Ok(())
}

/// Trigger immediate rollback to previous firmware.
#[cfg(target_os = "espidf")]
pub fn rollback_and_reboot() -> ! {
    log::warn!("OTA: triggering rollback and reboot");
    unsafe {
        esp_idf_sys::esp_ota_mark_app_invalid_rollback_and_reboot();
    }
    // Should not reach here — the above function reboots.
    unreachable!()
}

/// Download firmware from URL and flash to the inactive OTA partition.
/// Returns Ok(()) on success, after which a reboot will boot the new firmware.
#[cfg(target_os = "espidf")]
pub fn download_and_flash(url: &str) -> panman_core::error::Result<()> {
    use std::ffi::CString;

    log::info!("OTA: downloading firmware from {}", url);

    let url_c = CString::new(url).map_err(|e| PanmanError::Ota(e.to_string()))?;

    unsafe {
        // Get the next OTA partition
        let update_partition =
            esp_idf_sys::esp_ota_get_next_update_partition(core::ptr::null());
        if update_partition.is_null() {
            return Err(PanmanError::Ota("no OTA update partition found".into()));
        }

        let part_ref = &*update_partition;
        log::info!(
            "OTA: writing to partition '{}' at offset 0x{:x}",
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                part_ref.label.as_ptr() as *const u8,
                part_ref
                    .label
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(part_ref.label.len()),
            )),
            part_ref.address,
        );

        // Begin OTA update
        let mut ota_handle: esp_idf_sys::esp_ota_handle_t = 0;
        check_esp_ota(
            esp_idf_sys::esp_ota_begin(
                update_partition,
                esp_idf_sys::OTA_SIZE_UNKNOWN as usize,
                &mut ota_handle,
            ),
            "esp_ota_begin",
        )?;

        // Set up HTTP client
        let mut http_config: esp_idf_sys::esp_http_client_config_t = core::mem::zeroed();
        http_config.url = url_c.as_ptr();
        http_config.timeout_ms = 30000;
        http_config.buffer_size = 4096;

        let client = esp_idf_sys::esp_http_client_init(&http_config);
        if client.is_null() {
            esp_idf_sys::esp_ota_abort(ota_handle);
            return Err(PanmanError::Ota("failed to init HTTP client".into()));
        }

        // Open connection
        let ret = esp_idf_sys::esp_http_client_open(client, 0);
        if ret != esp_idf_sys::ESP_OK {
            esp_idf_sys::esp_http_client_cleanup(client);
            esp_idf_sys::esp_ota_abort(ota_handle);
            return Err(PanmanError::Ota(format!(
                "HTTP open failed: 0x{:x}",
                ret
            )));
        }

        let content_length = esp_idf_sys::esp_http_client_fetch_headers(client);
        log::info!("OTA: firmware size: {} bytes", content_length);

        // Read and write firmware chunks
        let mut buf = [0u8; 4096];
        let mut total_read: usize = 0;

        loop {
            let read_len = esp_idf_sys::esp_http_client_read(
                client,
                buf.as_mut_ptr() as *mut core::ffi::c_char,
                buf.len() as i32,
            );
            if read_len < 0 {
                esp_idf_sys::esp_http_client_cleanup(client);
                esp_idf_sys::esp_ota_abort(ota_handle);
                return Err(PanmanError::Ota("HTTP read error".into()));
            }
            if read_len == 0 {
                break;
            }

            check_esp_ota(
                esp_idf_sys::esp_ota_write(
                    ota_handle,
                    buf.as_ptr() as *const core::ffi::c_void,
                    read_len as usize,
                ),
                "esp_ota_write",
            )
            .map_err(|e| {
                esp_idf_sys::esp_http_client_cleanup(client);
                esp_idf_sys::esp_ota_abort(ota_handle);
                e
            })?;

            total_read += read_len as usize;
            if total_read % (64 * 1024) == 0 {
                log::info!("OTA: {} KB written", total_read / 1024);
            }
        }

        esp_idf_sys::esp_http_client_cleanup(client);

        log::info!("OTA: download complete, {} bytes total", total_read);

        // Finalize OTA
        check_esp_ota(esp_idf_sys::esp_ota_end(ota_handle), "esp_ota_end")?;

        // Set boot partition to the newly written one
        check_esp_ota(
            esp_idf_sys::esp_ota_set_boot_partition(update_partition),
            "esp_ota_set_boot_partition",
        )?;

        log::info!("OTA: firmware flashed successfully, reboot to activate");
    }

    Ok(())
}

/// Check for OTA update by fetching the manifest from the given URL.
/// Returns the manifest JSON body as bytes if successful.
#[cfg(target_os = "espidf")]
pub fn fetch_manifest(url: &str) -> panman_core::error::Result<Vec<u8>> {
    use std::ffi::CString;

    let url_c = CString::new(url).map_err(|e| PanmanError::Ota(e.to_string()))?;

    unsafe {
        let mut http_config: esp_idf_sys::esp_http_client_config_t = core::mem::zeroed();
        http_config.url = url_c.as_ptr();
        http_config.timeout_ms = 10000;

        let client = esp_idf_sys::esp_http_client_init(&http_config);
        if client.is_null() {
            return Err(PanmanError::Ota("failed to init HTTP client".into()));
        }

        let ret = esp_idf_sys::esp_http_client_perform(client);
        if ret != esp_idf_sys::ESP_OK {
            esp_idf_sys::esp_http_client_cleanup(client);
            return Err(PanmanError::Ota(format!(
                "HTTP request failed: 0x{:x}",
                ret
            )));
        }

        let status = esp_idf_sys::esp_http_client_get_status_code(client);
        if status != 200 {
            esp_idf_sys::esp_http_client_cleanup(client);
            return Err(PanmanError::Ota(format!("HTTP status: {}", status)));
        }

        let content_length = esp_idf_sys::esp_http_client_get_content_length(client);
        let mut body = vec![0u8; content_length.max(0) as usize + 1];
        let read_len = esp_idf_sys::esp_http_client_read_response(
            client,
            body.as_mut_ptr() as *mut core::ffi::c_char,
            body.len() as i32,
        );

        esp_idf_sys::esp_http_client_cleanup(client);

        if read_len < 0 {
            return Err(PanmanError::Ota("failed to read HTTP response".into()));
        }
        body.truncate(read_len as usize);
        Ok(body)
    }
}

/// Reboot the device.
#[cfg(target_os = "espidf")]
pub fn reboot() -> ! {
    log::info!("Rebooting...");
    unsafe {
        esp_idf_sys::esp_restart();
    }
    #[allow(unreachable_code)]
    loop {}
}

// ---- Host stubs ----

#[cfg(not(target_os = "espidf"))]
pub fn is_pending_verification() -> bool {
    false
}

#[cfg(not(target_os = "espidf"))]
pub fn confirm_firmware() -> panman_core::error::Result<()> {
    log::info!("OTA confirm: stub (host build)");
    Ok(())
}

#[cfg(not(target_os = "espidf"))]
pub fn fetch_manifest(_url: &str) -> panman_core::error::Result<Vec<u8>> {
    Err(PanmanError::Ota("OTA not available on host".into()))
}

#[cfg(not(target_os = "espidf"))]
pub fn download_and_flash(_url: &str) -> panman_core::error::Result<()> {
    Err(PanmanError::Ota("OTA not available on host".into()))
}
