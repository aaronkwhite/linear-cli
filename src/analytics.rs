use std::fs;
use std::path::Path;

/// Write-only PostHog project token. Safe to embed — cannot read data, only send events.
/// Alternative: use compile-time env var instead:
///   const POSTHOG_TOKEN: &str = env!("LIN_POSTHOG_TOKEN");
/// This requires LIN_POSTHOG_TOKEN set at build time (add to CI secrets).
const POSTHOG_TOKEN: &str = "phc_PLACEHOLDER";

const POSTHOG_BATCH_URL: &str = "https://app.posthog.com/batch/";

/// Check if analytics are enabled. Checks in order:
/// 1. DO_NOT_TRACK=1 env var → disabled
/// 2. Config analytics_enabled == Some(false) → disabled
/// 3. Otherwise → enabled
pub fn is_enabled() -> bool {
    if std::env::var("DO_NOT_TRACK").ok().as_deref() == Some("1") {
        return false;
    }
    crate::config::is_analytics_enabled()
}

/// Get or create the anonymous install ID. Returns (id, was_first_run).
/// Uses the default config dir.
fn get_or_create_install_id() -> Option<(String, bool)> {
    let dir = crate::config::config_dir()?;
    get_or_create_install_id_in(&dir)
}

/// Get or create install ID in a specific directory. Testable.
fn get_or_create_install_id_in(dir: &Path) -> Option<(String, bool)> {
    let path = dir.join("analytics_id");
    if let Ok(id) = fs::read_to_string(&path) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return Some((id, false));
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    fs::create_dir_all(dir).ok()?;
    fs::write(&path, &id).ok()?;
    Some((id, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enabled_default() {
        temp_env::with_var_unset("DO_NOT_TRACK", || {
            // With no config file and no env var, should default to enabled
            assert!(is_enabled());
        });
    }

    #[test]
    fn test_is_enabled_do_not_track() {
        temp_env::with_var("DO_NOT_TRACK", Some("1"), || {
            assert!(!is_enabled());
        });
    }

    #[test]
    fn test_is_enabled_do_not_track_other_values() {
        temp_env::with_var("DO_NOT_TRACK", Some("0"), || {
            assert!(is_enabled());
        });
        temp_env::with_var("DO_NOT_TRACK", Some("true"), || {
            assert!(is_enabled());
        });
    }

    #[test]
    fn test_install_id_created_on_first_run() {
        let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);

        let (id, first_run) = get_or_create_install_id_in(&dir).unwrap();
        assert!(first_run);
        assert_eq!(id.len(), 36); // UUID format

        // File should exist now
        let stored = fs::read_to_string(dir.join("analytics_id")).unwrap();
        assert_eq!(stored, id);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_id_loaded_on_second_run() {
        let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);

        let (id1, first) = get_or_create_install_id_in(&dir).unwrap();
        assert!(first);

        let (id2, second) = get_or_create_install_id_in(&dir).unwrap();
        assert!(!second);
        assert_eq!(id1, id2);

        let _ = fs::remove_dir_all(&dir);
    }
}
