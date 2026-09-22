//! Git hook installation logic.
//!
//! Implements idempotent `init_hooks()` that installs jig wrapper scripts
//! into `.git/hooks/`, backing up existing user hooks.

use std::path::Path;

use super::git::{generate_hook, is_jig_managed, MANAGED_HOOKS};
use super::registry::HookRegistry;

/// What to do for a given hook during init.
#[derive(Debug, PartialEq)]
pub enum InstallDecision {
    /// No existing hook — install fresh.
    Install,
    /// Already jig-managed and in registry — skip.
    Skip,
    /// Force flag — reinstall regardless.
    Reinstall,
    /// Jig-managed on disk but missing from registry — re-register.
    UpdateRegistry,
    /// User hook exists — back up and install.
    BackupAndInstall,
}

/// Outcome for a single hook.
#[derive(Debug)]
pub enum HookResult {
    Installed(String),
    AlreadyInstalled(String),
    BackedUpAndInstalled { hook: String, backup: String },
}

/// Aggregate result of `init_hooks`.
#[derive(Debug)]
pub struct InitResult {
    pub results: Vec<HookResult>,
}

/// Decide what to do for a single hook.
pub fn should_install_hook(
    hook_path: &Path,
    registry: &HookRegistry,
    hook_name: &str,
    force: bool,
) -> Result<InstallDecision, super::HookError> {
    if force {
        return Ok(InstallDecision::Reinstall);
    }
    if !hook_path.exists() {
        return Ok(InstallDecision::Install);
    }
    let content = std::fs::read_to_string(hook_path)?;
    if is_jig_managed(&content) {
        if registry.is_installed(hook_name) {
            return Ok(InstallDecision::Skip);
        } else {
            return Ok(InstallDecision::UpdateRegistry);
        }
    }
    Ok(InstallDecision::BackupAndInstall)
}

/// Install jig git hooks into `<repo_path>/.git/hooks/`.
///
/// Registry is saved at `<repo_path>/.jig/hooks/hooks.json`.
pub fn init_hooks(repo_path: &Path, force: bool) -> Result<InitResult, super::HookError> {
    let hooks_dir = repo_path.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let mut registry = HookRegistry::load(repo_path)?;
    let mut results = Vec::new();

    for hook_name in MANAGED_HOOKS {
        let hook_path = hooks_dir.join(hook_name);
        let decision = should_install_hook(&hook_path, &registry, hook_name, force)?;

        match decision {
            InstallDecision::Skip => {
                results.push(HookResult::AlreadyInstalled(hook_name.to_string()));
                continue;
            }
            InstallDecision::BackupAndInstall => {
                let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let backup_name = format!("{}.backup-{}", hook_name, date);
                let backup_path = hooks_dir.join(&backup_name);
                std::fs::copy(&hook_path, &backup_path)?;

                let user_path = hooks_dir.join(format!("{}.user", hook_name));
                std::fs::rename(&hook_path, &user_path)?;

                // Write the jig wrapper
                let content = generate_hook(hook_name)?;
                std::fs::write(&hook_path, &content)?;
                make_executable(&hook_path)?;

                registry.mark_installed(hook_name);
                registry.mark_existing_backed_up(hook_name, &backup_name);

                results.push(HookResult::BackedUpAndInstalled {
                    hook: hook_name.to_string(),
                    backup: backup_name,
                });
            }
            InstallDecision::UpdateRegistry
            | InstallDecision::Install
            | InstallDecision::Reinstall => {
                let content = generate_hook(hook_name)?;
                std::fs::write(&hook_path, &content)?;
                make_executable(&hook_path)?;
                registry.mark_installed(hook_name);
                results.push(HookResult::Installed(hook_name.to_string()));
            }
        }
    }

    registry.save(repo_path)?;
    Ok(InitResult { results })
}

fn make_executable(_path: &Path) -> Result<(), super::HookError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(_path, perms)?;
    }
    Ok(())
}
