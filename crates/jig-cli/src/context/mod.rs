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

pub use config::Config;
pub use config::{LinearConfig, LinearProfile, NotifyConfig};
pub use paths::{hook_registry_path, AppPaths};
pub use registry::{RepoEntry, RepoRegistry};
pub use repo::{
    AgentConfig, IssuesConfig, JigToml, LinearIssuesConfig, SpawnConfig, TriageConfig,
    WorktreeConfig,
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

/// Per-repo configuration: paths + jig.toml.
pub struct RepoConfig {
    pub repo_root: PathBuf,
    pub worktrees_path: PathBuf,
    pub git_common_dir: PathBuf,
    pub repo: JigToml,
}

impl RepoConfig {
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

    fn build(repo_root: PathBuf, git_common_dir: PathBuf) -> Result<Self, ContextError> {
        let worktrees_path = repo_root.join(JIG_DIR);
        let repo = JigToml::load(&repo_root)?.unwrap_or_default();
        Ok(Self {
            repo_root,
            worktrees_path,
            git_common_dir,
            repo,
        })
    }

    /// Effective base branch: jig.toml > global config > "origin/main"
    pub fn base_branch(&self, config: &Config) -> Branch {
        let name = self
            .repo
            .worktree
            .base
            .clone()
            .or_else(|| config.default_base_branch.clone())
            .unwrap_or_else(|| DEFAULT_BASE_BRANCH.to_string());
        Branch::new(name)
    }

    /// Display name derived from the repo root directory.
    pub fn name(&self) -> String {
        self.repo_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Tmux session name for this repo.
    pub fn session_name(&self) -> String {
        let repo_name = self
            .repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        format!("jig-{}", repo_name)
    }

    /// Create an issue provider from whatever backend is configured.
    pub fn issue_provider(&self, config: &Config) -> Result<IssueProvider, ContextError> {
        if self.repo.issues.linear.is_some() {
            return Ok(IssueProvider::new(Box::new(self.linear_provider(config)?)));
        }
        Err(ContextError::Config(
            "no issue provider configured — add [issues.linear] to jig.toml".into(),
        ))
    }

    /// Create a Linear provider.
    pub fn linear_provider(&self, config: &Config) -> Result<LinearProvider, ContextError> {
        let linear_config = self.repo.issues.linear.as_ref().ok_or_else(|| {
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

/// Runtime context: config + repo registry + resolved repo configs.
pub struct Context {
    pub paths: AppPaths,
    pub config: Config,
    pub registry: RepoRegistry,
    pub repos: Vec<RepoConfig>,
}

impl Context {
    /// Single repo from cwd.
    pub fn from_cwd(paths: &AppPaths) -> Result<Self, ContextError> {
        let config = Config::load(paths).unwrap_or_default();
        let repo = RepoConfig::from_cwd()?;
        Ok(Self::for_repo(paths, repo, config))
    }

    /// A single-repo context for `repo`, recording it in the global registry
    /// so daemon/-g commands see it.
    ///
    /// Recording is best-effort — last-writer-wins under concurrent jig
    /// processes, which is acceptable for a path list, and a filesystem
    /// problem must not fail an unrelated command.
    fn for_repo(paths: &AppPaths, repo: RepoConfig, config: Config) -> Self {
        register_globally(paths, &repo.repo_root);
        let mut registry = RepoRegistry::default();
        registry.register(repo.repo_root.clone());
        Self {
            paths: paths.clone(),
            config,
            registry,
            repos: vec![repo],
        }
    }

    /// All tracked repos.
    pub fn from_global(paths: &AppPaths) -> Result<Self, ContextError> {
        let config = Config::load(paths).unwrap_or_default();
        let registry = RepoRegistry::load(paths)?;
        let repos = registry
            .repos()
            .iter()
            .filter(|e| e.path.exists())
            .filter_map(|e| RepoConfig::from_path(&e.path).ok())
            .collect();
        Ok(Self {
            paths: paths.clone(),
            config,
            registry,
            repos,
        })
    }

    /// Single repo convenience — errors if no repos.
    pub fn repo(&self) -> Result<&RepoConfig, ContextError> {
        self.repos.first().ok_or(ContextError::NotInGitRepo)
    }
}

/// Single-repo context: the repo discovered from cwd plus global config.
pub struct RepoCtx {
    pub paths: AppPaths,
    pub repo: RepoConfig,
    pub config: Config,
    pub jig_toml: JigToml,
}

impl RepoCtx {
    pub fn from_cwd(paths: &AppPaths) -> Result<Self, ContextError> {
        let config = Config::load(paths).unwrap_or_default();
        let repo = RepoConfig::from_cwd()?;
        let jig_toml = JigToml::load(&repo.repo_root)
            .ok()
            .flatten()
            .unwrap_or_default();
        register_globally(paths, &repo.repo_root);
        Ok(Self {
            paths: paths.clone(),
            repo,
            config,
            jig_toml,
        })
    }
}

impl From<RepoCtx> for Context {
    fn from(ctx: RepoCtx) -> Self {
        let mut registry = RepoRegistry::default();
        registry.register(ctx.repo.repo_root.clone());
        Context {
            paths: ctx.paths,
            config: ctx.config,
            registry,
            repos: vec![ctx.repo],
        }
    }
}

/// All-repos context: full registry plus config.
pub struct GlobalCtx {
    pub paths: AppPaths,
    pub config: Config,
    pub registry: RepoRegistry,
    pub repos: Vec<RepoConfig>,
}

impl GlobalCtx {
    pub fn load(paths: &AppPaths) -> Result<Self, ContextError> {
        let config = Config::load(paths).unwrap_or_default();
        let registry = RepoRegistry::load(paths)?;
        let repos = registry
            .repos()
            .iter()
            .filter(|e| e.path.exists())
            .filter_map(|e| RepoConfig::from_path(&e.path).ok())
            .collect();
        Ok(Self {
            paths: paths.clone(),
            config,
            registry,
            repos,
        })
    }
}

impl From<GlobalCtx> for Context {
    fn from(ctx: GlobalCtx) -> Self {
        Context {
            paths: ctx.paths,
            config: ctx.config,
            registry: ctx.registry,
            repos: ctx.repos,
        }
    }
}

/// Either single-repo or all-repos context (for commands with `--global`).
// CLI context enum — constructed once per invocation, not on a hot path.
#[allow(clippy::large_enum_variant)]
pub enum ScopedCtx {
    Repo(RepoCtx),
    Global(GlobalCtx),
}

impl ScopedCtx {
    pub fn paths(&self) -> &AppPaths {
        match self {
            ScopedCtx::Repo(r) => &r.paths,
            ScopedCtx::Global(g) => &g.paths,
        }
    }

    pub fn from_global(paths: &AppPaths, global: bool) -> Result<Self, ContextError> {
        if global {
            Ok(ScopedCtx::Global(GlobalCtx::load(paths)?))
        } else {
            Ok(ScopedCtx::Repo(RepoCtx::from_cwd(paths)?))
        }
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
pub fn resolve_base_branch_for(repo_root: &Path, config: &Config) -> Result<Branch, ContextError> {
    if let Ok(Some(jig_toml)) = JigToml::load(repo_root) {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::Sandbox;

    #[test]
    fn test_single_repo_context_registers_the_repo() {
        let sandbox = Sandbox::with_repos(1);
        let paths = sandbox.paths();
        let dir = sandbox.repo(0);

        let repo = RepoConfig::from_path(dir).unwrap();
        let ctx = Context::for_repo(&paths, repo, Config::default());

        let repo = ctx.repo().unwrap();
        assert_eq!(
            repo.repo_root.canonicalize().unwrap(),
            dir.canonicalize().unwrap()
        );
        assert!(repo.worktrees_path.ends_with(JIG_DIR));
        assert!(repo.session_name().starts_with("jig-"));
        assert_eq!(repo.base_branch(&ctx.config), "origin/main");

        // The context's own registry holds just this repo...
        assert_eq!(
            ctx.registry.repos().len(),
            1,
            "registry returned by from_cwd should contain the current repo"
        );
        assert_eq!(
            ctx.registry.repos()[0].path.canonicalize().unwrap(),
            dir.canonicalize().unwrap()
        );

        // ...and it is persisted globally so -g commands and the daemon see it.
        let global = RepoRegistry::load(&paths).unwrap();
        assert_eq!(global.repos().len(), 1);
        assert_eq!(
            global.repos()[0].path.canonicalize().unwrap(),
            dir.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_registry_no_duplicate_on_repeated_register() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().to_path_buf();

        let mut registry = RepoRegistry::default();
        let added_first = registry.register(repo_path.clone());
        let added_second = registry.register(repo_path.clone());

        assert!(added_first, "first register should report newly added");
        assert!(
            !added_second,
            "second register should report already present"
        );
        assert_eq!(
            registry.repos().len(),
            1,
            "repeated register must not duplicate entries"
        );
    }

    #[test]
    fn test_base_branch_from_jig_toml() {
        let sandbox = Sandbox::with_repos(1);
        let dir = sandbox.repo(0);
        std::fs::write(
            dir.join("jig.toml"),
            "[worktree]\nbase = \"origin/develop\"\n",
        )
        .unwrap();

        let repo = RepoConfig::from_path(dir).unwrap();
        assert_eq!(repo.base_branch(&Config::default()), "origin/develop");
    }
}
