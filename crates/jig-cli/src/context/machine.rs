//! Machine identity — which machine a daemon (and its workers) lives on.
//!
//! Two parts, deliberately separate:
//!
//! * **name** — for people. `[fleet] name` in the global config, else the
//!   hostname. Free to change; the fleet view labels rows with it.
//! * **node id** — for machines. Generated once and kept in
//!   `~/.config/jig/node_id`, so it survives daemon restarts, renames and
//!   hostname changes. Delete the file to mint a new identity.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use nix::unistd::gethostname;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::config::AppConfig;
use super::paths::AppPaths;

/// Used when neither config nor the OS gives us a usable name.
const UNKNOWN_HOST: &str = "unknown";

/// This machine, as a daemon reports it in `Ping` and `GetStatus`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    /// Human-readable machine name.
    pub name: String,
    /// Stable, generated-once identifier.
    pub node_id: Uuid,
}

impl Machine {
    /// Resolve the name and load the node id, minting it on first use.
    pub fn load(paths: &AppPaths, config: &AppConfig) -> io::Result<Self> {
        Ok(Self {
            name: machine_name(config.fleet.name.as_deref(), hostname()),
            node_id: load_or_create_node_id(&paths.node_id_file())?,
        })
    }
}

/// The configured name if it has any content, else the hostname.
fn machine_name(configured: Option<&str>, hostname: Option<String>) -> String {
    configured
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .or(hostname)
        .unwrap_or_else(|| UNKNOWN_HOST.to_string())
}

fn hostname() -> Option<String> {
    let name = gethostname().ok()?.into_string().ok()?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Read the node id at `path`, or generate and persist one if there is none.
///
/// A file that exists but does not hold a node id is an error, not a cue to
/// mint a new one: silently changing identity would orphan this machine's
/// rows in every peer's fleet view.
fn load_or_create_node_id(path: &Path) -> io::Result<Uuid> {
    match fs::read_to_string(path) {
        Ok(content) => Uuid::parse_str(content.trim()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} is not a node id ({e}) — delete it to mint a new one",
                    path.display()
                ),
            )
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => create_node_id(path),
        Err(e) => Err(e),
    }
}

/// Write a fresh node id with `create_new`, so two processes racing on first
/// use agree on whichever landed first.
fn create_node_id(path: &Path) -> io::Result<Uuid> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let id = Uuid::new_v4();
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => {
            writeln!(file, "{id}")?;
            Ok(id)
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => load_or_create_node_id(path),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(dir: &Path) -> AppPaths {
        AppPaths::under(&dir.join("config"), &dir.join("run"))
    }

    #[test]
    fn the_node_id_survives_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths(dir.path());
        let config = AppConfig::default();

        let first = Machine::load(&paths, &config).unwrap();
        let second = Machine::load(&paths, &config).unwrap();
        assert_eq!(first, second);
        assert!(paths.node_id_file().exists());
    }

    #[test]
    fn separate_config_dirs_get_separate_node_ids() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let config = AppConfig::default();
        assert_ne!(
            Machine::load(&paths(a.path()), &config).unwrap().node_id,
            Machine::load(&paths(b.path()), &config).unwrap().node_id
        );
    }

    #[test]
    fn a_corrupt_node_id_is_an_error_not_a_new_identity() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths(dir.path());
        fs::create_dir_all(paths.config_dir()).unwrap();
        fs::write(paths.node_id_file(), "not-a-uuid\n").unwrap();

        let err = Machine::load(&paths, &AppConfig::default()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            fs::read_to_string(paths.node_id_file()).unwrap(),
            "not-a-uuid\n"
        );
    }

    #[test]
    fn a_configured_name_wins_over_the_hostname() {
        let dir = tempfile::tempdir().unwrap();
        let config: AppConfig = toml::from_str("[fleet]\nname = \"devbox\"\n").unwrap();
        let machine = Machine::load(&paths(dir.path()), &config).unwrap();
        assert_eq!(machine.name, "devbox");
    }

    #[test]
    fn a_blank_name_falls_back_to_the_hostname() {
        assert_eq!(machine_name(Some("  "), Some("host".into())), "host");
        assert_eq!(machine_name(None, Some("host".into())), "host");
        assert_eq!(machine_name(Some(" box "), Some("host".into())), "box");
        assert_eq!(machine_name(None, None), UNKNOWN_HOST);
    }

    #[test]
    fn machine_round_trips_through_json() {
        let machine = Machine {
            name: "laptop".into(),
            node_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&machine).unwrap();
        assert_eq!(serde_json::from_str::<Machine>(&json).unwrap(), machine);
    }
}
