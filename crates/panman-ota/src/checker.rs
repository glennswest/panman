use panman_core::error::{PanmanError, Result};
use serde::{Deserialize, Serialize};

/// OTA firmware manifest returned by the update server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareManifest {
    pub version: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub release_notes: Option<String>,
}

/// OTA version checker.
pub struct OtaChecker {
    check_url: String,
    current_version: String,
}

impl OtaChecker {
    pub fn new(check_url: &str, current_version: &str) -> Self {
        Self {
            check_url: check_url.to_string(),
            current_version: current_version.to_string(),
        }
    }

    pub fn check_url(&self) -> &str {
        &self.check_url
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }

    /// Parse a manifest response and check if an update is available.
    pub fn check_manifest(&self, body: &[u8]) -> Result<Option<FirmwareManifest>> {
        let manifest: FirmwareManifest =
            serde_json::from_slice(body).map_err(|e| PanmanError::Ota(e.to_string()))?;

        if self.is_newer(&manifest.version) {
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    /// Simple version comparison: split on '.', compare each segment numerically.
    fn is_newer(&self, remote: &str) -> bool {
        let parse = |v: &str| -> Vec<u32> {
            v.trim_start_matches('v')
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect()
        };
        let current = parse(&self.current_version);
        let remote = parse(remote);

        for (r, c) in remote.iter().zip(current.iter()) {
            match r.cmp(c) {
                std::cmp::Ordering::Greater => return true,
                std::cmp::Ordering::Less => return false,
                std::cmp::Ordering::Equal => continue,
            }
        }
        remote.len() > current.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        let checker = OtaChecker::new("http://example.com/ota", "0.1.0");
        assert!(checker.is_newer("0.2.0"));
        assert!(checker.is_newer("0.1.1"));
        assert!(checker.is_newer("1.0.0"));
        assert!(!checker.is_newer("0.1.0"));
        assert!(!checker.is_newer("0.0.9"));
    }

    #[test]
    fn check_manifest_newer() {
        let checker = OtaChecker::new("http://example.com/ota", "0.1.0");
        let manifest = r#"{"version":"0.2.0","url":"http://example.com/fw.bin","sha256":"abc123"}"#;
        let result = checker.check_manifest(manifest.as_bytes()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, "0.2.0");
    }

    #[test]
    fn check_manifest_same() {
        let checker = OtaChecker::new("http://example.com/ota", "0.1.0");
        let manifest = r#"{"version":"0.1.0","url":"http://example.com/fw.bin","sha256":"abc123"}"#;
        let result = checker.check_manifest(manifest.as_bytes()).unwrap();
        assert!(result.is_none());
    }
}
