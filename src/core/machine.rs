//! Machine information capture for file metadata.
//!
//! Collects hostname and primary IP address at save time.

use crate::core::format::MachineInfo;

/// Collect machine identity for embedding in saved files.
///
/// Returns hostname and first non-loopback IPv4 address.
/// Falls back to "unknown" if detection fails — never blocks saving.
pub fn collect_machine_info() -> MachineInfo {
    let hostname = gethostname::gethostname().to_string_lossy().to_string();

    let ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    MachineInfo { hostname, ip }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_machine_info_returns_valid_structure() {
        let info = collect_machine_info();
        // Just verify it returns something valid; actual values depend on environment
        assert!(!info.hostname.is_empty());
        assert!(!info.ip.is_empty());
    }
}
