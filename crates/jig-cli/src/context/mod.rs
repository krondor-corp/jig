//! Context module — configuration, state, and runtime context.
//!
//! Three tiers of config:
//! - Global: `~/.config/jig/config.toml` (user-wide defaults)
//! - Repo committed: `jig.toml` (checked into the repo)
//! - Repo local: `jig.local.toml` (gitignored overrides, merged on top)
//!
//! `Context` composes config + repo registry + resolved repo configs.

pub mod config;
pub mod log;
pub mod paths;
pub mod registry;
pub mod repo;

use std::path::{Path, PathBuf};

use jig_core::git::{Branch, Repo};
use jig_core::issues::{IssueProvider, LinearProvider};

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Git(#[from] jig_core::git::GitError),
    #[error(transparent)]
    Linear(#[from] jig_core::issues::providers::linear::client::LinearError),
    #[error("not in a git repository")]
    NotInGitRepo,
    #[error("{0}")]
    Config(String),
}

pub use config::AppConfig;
pub use config::{GitConfig, LinearConfig, LinearProfile, NotifyConfig};
pub use paths::{hook_registry_path, AppPaths};
pub use registry::{RepoEntry, RepoRegistry};
pub use repo::{
    AgentConfig, IssuesConfig, LinearIssuesConfig, RepoConfig, TriageConfig, WorktreeConfig,
};

/// Directory name for jig-managed worktrees (relative to repo root)
pub const JIG_DIR: &str = ".jig";
/// Repo config file name
pub const JIG_TOML: &str = "jig.toml";
/// Local (gitignored) config overlay file name
pub const JIG_LOCAL_TOML: &str = "jig.local.toml";
/// Default base branch when nothing is configured
pub const DEFAULT_BASE_BRANCH: &str = "origin/main";

/// Build the worktree path for a worker within a repo root.
pub fn worktree_path(repo_root: &Path, worker_name: &str) -> PathBuf {
    repo_root.join(JIG_DIR).join(worker_name)
}

/// Where one repo lives on disk.
pub struct RepoPaths {
    pub repo_root: PathBuf,
    pub worktrees_path: PathBuf,
    pub git_common_dir: PathBuf,
}

/// A repo jig knows about: where it is, and how it is configured.
///
/// Mirrors [`AppCtx`] one level down — paths plus the config that governs
/// them.
pub struct RepoCtx {
    pub paths: RepoPaths,
    pub config: RepoConfig,
}

impl RepoCtx {
    pub fn from_cwd() -> Result<Self, ContextError> {
        let git_repo = Repo::discover()?;
        let git_common_dir = git_repo.common_dir();
        let repo_root = git_common_dir
            .parent()
            .unwrap_or(&git_common_dir)
            .to_path_buf();
        Self::build(repo_root, git_common_dir)
    }

    pub fn from_path(path: &Path) -> Result<Self, ContextError> {
        let git_repo = Repo::open(path)?;
        let git_common_dir = git_repo.common_dir();
        let repo_root = git_common_dir
            .parent()
            .unwrap_or(&git_common_dir)
            .to_path_buf();
        Self::build(repo_root, git_common_dir)
    }

    /// A missing `jig.toml` is fine and yields defaults. A malformed one is
    /// an error — silently falling back would drop the agent, issue provider
    /// and hooks the user thought they had configured.
    fn build(repo_root: PathBuf, git_common_dir: PathBuf) -> Result<Self, ContextError> {
        let worktrees_path = repo_root.join(JIG_DIR);
        let config = RepoConfig::load(&repo_root)?.unwrap_or_default();
        Ok(Self {
            paths: RepoPaths {
                repo_root,
                worktrees_path,
                git_common_dir,
            },
            config,
        })
    }

    /// Effective base branch: jig.toml > global config > "origin/main"
    pub fn base_branch(&self, config: &AppConfig) -> Branch {
        let name = self
            .config
            .worktree
            .base
            .clone()
            .or_else(|| config.default_base_branch.clone())
            .unwrap_or_else(|| DEFAULT_BASE_BRANCH.to_string());
        Branch::new(name)
    }

    /// Display name derived from the repo root directory.
    pub fn name(&self) -> String {
        self.paths
            .repo_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Tmux session name for this repo.
    pub fn session_name(&self) -> String {
        let repo_name = self
            .paths
            .repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        format!("jig-{}", repo_name)
    }

    /// Create an issue provider from whatever backend is configured.
    pub fn issue_provider(&self, config: &AppConfig) -> Result<IssueProvider, ContextError> {
        if self.config.issues.linear.is_some() {
            return Ok(IssueProvider::new(Box::new(self.linear_provider(config)?)));
        }
        Err(ContextError::Config(
            "no issue provider configured — add [issues.linear] to jig.toml".into(),
        ))
    }

    /// Create a Linear provider.
    pub fn linear_provider(&self, config: &AppConfig) -> Result<LinearProvider, ContextError> {
        let linear_config = self.config.issues.linear.as_ref().ok_or_else(|| {
            ContextError::Config(
                "[issues.linear] config required when provider = \"linear\"".into(),
            )
        })?;

        let profile = config
            .linear
            .profiles
            .get(&linear_config.profile)
            .ok_or_else(|| {
                ContextError::Config(format!(
                    "Linear profile '{}' not found in global config (~/.config/jig/config.toml)",
                    linear_config.profile,
                ))
            })?;

        let team = linear_config
            .team
            .clone()
            .or_else(|| profile.team.clone())
            .ok_or_else(|| {
                ContextError::Config(
                    "Linear team key is required — set 'team' in [issues.linear] in jig.toml or in the profile in ~/.config/jig/config.toml"
                        .to_string(),
                )
            })?;

        let projects = if linear_config.projects.is_empty() {
            profile.projects.clone()
        } else {
            linear_config.projects.clone()
        };

        let labels = if linear_config.labels.is_empty() {
            profile.labels.clone()
        } else {
            linear_config.labels.clone()
        };

        let assignee = linear_config
            .assignee
            .clone()
            .or_else(|| profile.assignee.clone());

        Ok(LinearProvider::new(
            &profile.api_key,
            team,
            projects,
            assignee,
            labels,
        )?)
    }
}

/// Flags that apply to every command, from the top-level CLI.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub verbose: bool,
    pub plain: bool,
}

/// Everything resolvable before we know which command this is: where jig's
/// files are, what the user configured, and how they invoked us.
///
/// Built once in `main` and handed to every command's `build_context`.
pub struct AppCtx {
    pub paths: AppPaths,
    pub config: AppConfig,
    pub flags: Flags,
}

impl AppCtx {
    /// A malformed global config falls back to defaults rather than failing,
    /// because `jig config` is how you would fix it — but it says so, rather
    /// than pretending the file was empty.
    pub fn load(paths: AppPaths, flags: Flags) -> Self {
        let config = match AppConfig::load(&paths) {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!("ignoring {}: {e}", paths.config_file().display());
                AppConfig::default()
            }
        };
        Self {
            paths,
            config,
            flags,
        }
    }

    /// Process-wide state, set once.
    ///
    /// Everything here writes a global — a C library's settings, or a static
    /// in this process — so it happens in one place rather than scattered
    /// down `main`. Must run before any thread is spawned.
    pub fn set_globals(&self) {
        crate::cli::ui::set_plain(self.flags.plain);

        // The `colored` crate checks stdout for TTY detection, but all jig
        // output goes to stderr. Decide from stderr instead.
        if !self.flags.plain && std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            colored::control::set_override(true);
        }
    }
}

/// What a command runs against.
///
/// `repos` holds the repo containing the current directory, or every tracked
/// repo under `-g`. That difference is the whole of what the scope means:
/// commands that iterate do not need to know which they were given.
pub struct Ctx {
    pub paths: AppPaths,
    pub config: AppConfig,
    pub flags: Flags,
    pub registry: RepoRegistry,
    repos: Vec<RepoCtx>,
}

impl Ctx {
    /// The repo containing the current directory.
    ///
    /// Records it in the global registry, so `-g` commands and the daemon
    /// can see a repo you have used at least once. Best effort — last writer
    /// wins, which is fine for a path list, and a filesystem problem must not
    /// fail an unrelated command.
    pub fn here(app: AppCtx) -> Result<Self, ContextError> {
        let repo = RepoCtx::from_cwd()?;
        register_globally(&app.paths, &repo.paths.repo_root);
        let mut registry = RepoRegistry::default();
        registry.register(repo.paths.repo_root.clone());
        Ok(Self {
            paths: app.paths,
            config: app.config,
            flags: app.flags,
            registry,
            repos: vec![repo],
        })
    }

    /// Every repo in the registry whose directory still exists.
    pub fn everywhere(app: AppCtx) -> Result<Self, ContextError> {
        let registry = RepoRegistry::load(&app.paths)?;
        let repos = registry
            .repos()
            .iter()
            .filter(|e| e.path.exists())
            .filter_map(|e| RepoCtx::from_path(&e.path).ok())
            .collect();
        Ok(Self {
            paths: app.paths,
            config: app.config,
            flags: app.flags,
            registry,
            repos,
        })
    }

    /// What `-g` means.
    pub fn scoped(app: AppCtx, global: bool) -> Result<Self, ContextError> {
        if global {
            Self::everywhere(app)
        } else {
            Self::here(app)
        }
    }

    /// A context for one already-resolved repo.
    pub fn for_repo(app: AppCtx, repo: RepoCtx) -> Self {
        register_globally(&app.paths, &repo.paths.repo_root);
        let mut registry = RepoRegistry::default();
        registry.register(repo.paths.repo_root.clone());
        Self {
            paths: app.paths,
            config: app.config,
            flags: app.flags,
            registry,
            repos: vec![repo],
        }
    }

    pub fn repos(&self) -> &[RepoCtx] {
        &self.repos
    }

    /// The repo this command acts on — for commands that only make sense in
    /// one, which are the ones built with [`Ctx::here`].
    pub fn repo(&self) -> Result<&RepoCtx, ContextError> {
        self.repos.first().ok_or(ContextError::NotInGitRepo)
    }
}

/// Update a key in the local (gitignored) `jig.local.toml` config.
///
/// Pass `Some(value)` to set, `None` to remove. Removes empty sections.
pub fn update_local_toml(
    repo_root: &Path,
    section: &str,
    key: &str,
    value: Option<&str>,
) -> Result<(), ContextError> {
    let local_path = repo_root.join(JIG_LOCAL_TOML);
    let mut doc: toml::Value = if local_path.exists() {
        let content = std::fs::read_to_string(&local_path)?;
        toml::from_str(&content).map_err(|e| ContextError::Config(e.to_string()))?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let table = doc.as_table_mut().unwrap();

    match value {
        Some(v) => {
            let section_table = table
                .entry(section)
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
                .as_table_mut()
                .ok_or_else(|| ContextError::Config(format!("[{}] is not a table", section)))?;
            section_table.insert(key.to_string(), toml::Value::String(v.to_string()));
        }
        None => {
            if let Some(section_val) = table.get_mut(section) {
                if let Some(section_table) = section_val.as_table_mut() {
                    section_table.remove(key);
                    if section_table.is_empty() {
                        table.remove(section);
                    }
                }
            }
        }
    }

    let content = toml::to_string_pretty(&doc).map_err(|e| ContextError::Config(e.to_string()))?;
    std::fs::write(&local_path, content)?;

    Ok(())
}

/// Record `repo_root` in the global registry so daemon/-g commands see it.
/// Best-effort: a filesystem problem must not fail an unrelated command.
fn register_globally(paths: &AppPaths, repo_root: &Path) {
    let mut global = RepoRegistry::load(paths).unwrap_or_default();
    global.register(repo_root.to_path_buf());
    let _ = global.save(paths);
}

/// Resolve the effective base branch for an arbitrary repo path
/// (without building a full Context). Used by daemon code.
pub fn resolve_base_branch_for(
    repo_root: &Path,
    config: &AppConfig,
) -> Result<Branch, ContextError> {
    if let Ok(Some(jig_toml)) = RepoConfig::load(repo_root) {
        if let Some(base) = jig_toml.worktree.base {
            return Ok(Branch::new(base));
        }
    }
    Ok(Branch::new(
        config
            .default_base_branch
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_BRANCH.to_string()),
    ))
}
