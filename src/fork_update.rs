//! Fork-only update helpers: ordering of fork releases that share a base
//! version, and detection of winget-managed installs.
//!
//! Kept out of `update.rs` so upstream merges stay small.

use std::path::Path;

pub(crate) const WINGET_UPDATE_COMMAND: &str = "winget upgrade khanhtd36.herdr-khanhtd36";

const FORK_SUFFIX_PREFIX: &str = "khanhtd36.";

/// Counter after `-khanhtd36.` in a version string; 0 when absent or malformed.
fn fork_counter(version: &str) -> u32 {
    version
        .split_once('-')
        .and_then(|(_, suffix)| suffix.strip_prefix(FORK_SUFFIX_PREFIX))
        .and_then(|counter| counter.parse().ok())
        .unwrap_or(0)
}

/// True when `latest` is a newer fork release than `current` on the same base
/// version. A higher base version is handled by the caller's triple compare.
pub(crate) fn is_newer_fork_release(latest: &str, current: &str) -> bool {
    let strip = |s: &str| s.trim().trim_start_matches('v').to_string();
    let (latest, current) = (strip(latest), strip(current));
    let base = |s: &str| {
        s.split_once('-')
            .map_or(s.to_string(), |(b, _)| b.to_string())
    };
    base(&latest) == base(&current) && fork_counter(&latest) > fork_counter(&current)
}

/// True for a binary installed by winget (portable package or its link).
pub(crate) fn is_winget_exe_path(path: &Path) -> bool {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    normalized.contains("/microsoft/winget/packages/")
        || normalized.contains("/microsoft/winget/links/")
}

pub(crate) fn is_winget_managed_install() -> bool {
    std::env::current_exe().is_ok_and(|exe| is_winget_exe_path(&exe))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_counter_on_same_base_is_newer() {
        assert!(is_newer_fork_release(
            "0.9.5-khanhtd36.2",
            "0.9.5-khanhtd36.1"
        ));
        assert!(is_newer_fork_release(
            "v0.9.5-khanhtd36.10",
            "0.9.5-khanhtd36.9"
        ));
        assert!(is_newer_fork_release("0.9.5-khanhtd36.1", "0.9.5"));
    }

    #[test]
    fn same_older_or_other_base_is_not_newer() {
        assert!(!is_newer_fork_release(
            "0.9.5-khanhtd36.1",
            "0.9.5-khanhtd36.1"
        ));
        assert!(!is_newer_fork_release(
            "0.9.5-khanhtd36.1",
            "0.9.5-khanhtd36.2"
        ));
        assert!(!is_newer_fork_release("0.9.5", "0.9.5-khanhtd36.1"));
        // Different base is the caller's triple compare, not ours.
        assert!(!is_newer_fork_release(
            "0.9.4-khanhtd36.9",
            "0.9.5-khanhtd36.1"
        ));
    }

    #[test]
    fn foreign_or_malformed_suffix_counts_as_zero() {
        assert_eq!(fork_counter("0.9.5"), 0);
        assert_eq!(fork_counter("0.9.5-preview.3"), 0);
        assert_eq!(fork_counter("0.9.5-khanhtd36.x"), 0);
        assert_eq!(fork_counter("0.9.5-khanhtd36.4"), 4);
    }

    #[test]
    fn winget_paths_are_detected() {
        for p in [
            r"C:\Users\u\AppData\Local\Microsoft\WinGet\Packages\khanhtd36.herdr-khanhtd36_Microsoft.Winget.Source_8wekyb3d8bbwe\herdr.exe",
            r"C:\Users\u\AppData\Local\Microsoft\WinGet\Links\herdr.exe",
            "c:/users/u/appdata/local/microsoft/winget/links/herdr.exe",
        ] {
            assert!(is_winget_exe_path(Path::new(p)), "{p}");
        }
        for p in [
            r"C:\Users\u\AppData\Local\Programs\Herdr\bin\herdr.exe",
            "/usr/local/bin/herdr",
        ] {
            assert!(!is_winget_exe_path(Path::new(p)), "{p}");
        }
    }
}
