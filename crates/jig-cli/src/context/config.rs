//! Global configuration — `~/.config/jig/config.toml`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::paths::AppPaths;
use super::ContextError;

/// Notification configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NotifyConfig {
    pub exec: Option<String>,
    pub webhook: Option<String>,
    pub events: Vec<String>,
}

/// Linear API configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LinearConfig {
    pub profiles: HashMap<String, LinearProfile>,
}

/// A single Linear API profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearProfile {
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// Global configuration stored at `~/.config/jig/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // Health
    pub silence_threshold_seconds: u64,

    // Spawn
    pub max_concurrent_workers: usize,

    // Daemon
    pub auto_recover: bool,
    pub tick_interval: u64,
    pub poll_interval: u64,
    pub session_prefix: String,

    // Mux backend hosting worker terminals ("tmux" or "herdr").
    // JIG_MUX env var overrides for one-off runs.
    pub mux: jig_core::mux::MuxKind,

    // GitHub
    pub auto_cleanup_merged: bool,
    pub auto_cleanup_closed: bool,

    #[serde(default)]
    pub notify: NotifyConfig,

    #[serde(default)]
    pub linear: LinearConfig,

    pub default_base_branch: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            silence_threshold_seconds: 300,
            max_concurrent_workers: 3,
            poll_interval: 120,
            auto_recover: true,
            tick_interval: 30,
            session_prefix: "jig-".to_string(),
            mux: jig_core::mux::MuxKind::default(),
            auto_cleanup_merged: true,
            auto_cleanup_closed: false,
            notify: NotifyConfig::default(),
            linear: LinearConfig::default(),
            default_base_branch: None,
        }
    }
}

impl Config {
    pub fn load(paths: &AppPaths) -> Result<Self, ContextError> {
        Self::load_from(&paths.config_file())
    }

    pub fn load_from(path: &Path) -> Result<Self, ContextError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, paths: &AppPaths) -> Result<(), ContextError> {
        self.save_to(&paths.config_file())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), ContextError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| ContextError::Config(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Initialize global config at ~/.config/jig/config.toml.
    /// Returns the path if created, None if it already exists (and force is false).
    pub fn init(paths: &AppPaths, force: bool) -> Result<Option<std::path::PathBuf>, ContextError> {
        let config_dir = paths.config_dir();
        let config_path = paths.config_file();

        if config_path.exists() && !force {
            return Ok(None);
        }

        fs::create_dir_all(config_dir)?;

        let content = r#"# jig global configuration

[health]
silence_threshold_seconds = 300  # seconds of silence before worker is "stalled"

[github]
auto_cleanup_merged = true       # clean up workers when PR merges
auto_cleanup_closed = false      # clean up workers when PR closed without merge

[spawn]
max_concurrent_workers = 3       # max auto-spawned workers per repo
poll_interval = 120              # seconds between issue polls

# [notify]
# exec = "~/.config/jig/hooks/notify.sh"
# events = ["needs_intervention", "worker_failed"]

# [linear.profiles.work]
# api_key = "lin_api_xxxxxxxxxxxx"
# team = "ENG"
"#;

        fs::write(&config_path, content)?;
        Ok(Some(config_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let cfg = Config::default();
        assert_eq!(cfg.silence_threshold_seconds, 300);
        assert_eq!(cfg.max_concurrent_workers, 3);
        assert_eq!(cfg.poll_interval, 120);
        assert!(cfg.auto_recover);
        assert_eq!(cfg.tick_interval, 30);
        assert!(cfg.notify.exec.is_none());
        assert!(cfg.default_base_branch.is_none());
    }
}
