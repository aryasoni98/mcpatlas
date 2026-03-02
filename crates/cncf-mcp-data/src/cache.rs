use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::models::Project;

/// Default cache file name.
const CACHE_FILE: &str = "cncf_landscape_cache.json";

/// Wrapper that stores projects alongside metadata for staleness checks.
#[derive(Serialize, Deserialize)]
struct CacheEnvelope {
    /// Unix timestamp (seconds) when the cache was written.
    created_at: u64,
    /// Number of projects (sanity check).
    count: usize,
    /// The cached project data.
    projects: Vec<Project>,
}

/// Resolve the cache file path inside the given directory.
pub fn cache_path(dir: &Path) -> PathBuf {
    dir.join(CACHE_FILE)
}

/// Try to load projects from a local cache file.
///
/// Returns `None` if the cache file doesn't exist, is unreadable, or is older
/// than `max_age`.
pub fn load(dir: &Path, max_age: Duration) -> Option<Vec<Project>> {
    let path = cache_path(dir);
    let data = std::fs::read_to_string(&path).ok()?;
    let envelope: CacheEnvelope = serde_json::from_str(&data).ok()?;

    // Check staleness
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let age_secs = now.saturating_sub(envelope.created_at);

    if max_age.is_zero() || age_secs > max_age.as_secs() {
        info!(
            "Cache expired (age {}s > max {}s), will refresh",
            age_secs,
            max_age.as_secs()
        );
        return None;
    }

    info!(
        "Loaded {} projects from cache (age {}s)",
        envelope.count, age_secs
    );
    Some(envelope.projects)
}

/// Write projects to the cache file. Creates the directory if needed.
pub fn save(dir: &Path, projects: &[Project]) -> Result<()> {
    std::fs::create_dir_all(dir).context("creating cache directory")?;

    let envelope = CacheEnvelope {
        created_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        count: projects.len(),
        projects: projects.to_vec(),
    };

    let path = cache_path(dir);
    let json = serde_json::to_string(&envelope).context("serializing cache")?;
    std::fs::write(&path, json).context("writing cache file")?;

    info!(
        "Saved {} projects to cache at {}",
        projects.len(),
        path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;

    fn dummy_project(name: &str) -> Project {
        Project {
            name: name.to_string(),
            description: None,
            homepage_url: None,
            repo_url: None,
            logo: None,
            crunchbase: None,
            category: String::new(),
            subcategory: String::new(),
            maturity: None,
            extra: Default::default(),
            github: None,
            artifact_hub_packages: None,
            summary: None,
            summary_content_hash: None,
        }
    }

    #[test]
    fn test_round_trip() {
        let dir = std::env::temp_dir().join("cncf_cache_test_rt");
        let _ = std::fs::remove_dir_all(&dir);

        let projects = vec![dummy_project("Kubernetes"), dummy_project("Prometheus")];
        save(&dir, &projects).unwrap();

        let loaded = load(&dir, Duration::from_secs(3600)).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Kubernetes");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_expired_cache() {
        let dir = std::env::temp_dir().join("cncf_cache_test_exp");
        let _ = std::fs::remove_dir_all(&dir);

        let projects = vec![dummy_project("Envoy")];
        save(&dir, &projects).unwrap();

        // Max age of 0 means always expired
        let loaded = load(&dir, Duration::from_secs(0));
        assert!(loaded.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_missing_cache() {
        let dir = std::env::temp_dir().join("cncf_cache_test_missing");
        let _ = std::fs::remove_dir_all(&dir);

        let loaded = load(&dir, Duration::from_secs(3600));
        assert!(loaded.is_none());
    }
}
