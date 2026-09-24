//! Git hook uninstall and rollback logic.

use std::path::Path;

use super::registry::{self, HookRegistry};

/// Outcome for a single hook uninstall.
#[derive(Debug)]
pub enum UninstallOutcome {
    /// Hook removed, no previous hook to restore.
    Removed(String),
    /// Restored from backup file.
    RestoredBackup { hook: String, backup: String },
    /// Restored from `.user` suffix.
    RestoredUser(String),
}

/// Aggregate result of `uninstall_hooks`.
#[derive(Debug)]
pub struct UninstallResult {
    pub outcomes: Vec<UninstallOutcome>,
}

/// Uninstall jig git hooks from the repo.
///
/// If `specific_hook` is `Some`, only that hook is removed.
/// Otherwise all hooks tracked in the registry are removed.
pub fn uninstall_hooks(
    repo_path: &Path,
    specific_hook: Option<&str>,
) -> Result<UninstallResult, super::HookError> {
    let hooks_dir = repo_path.join(".git").join("hooks");
    let registry_path = registry::registry_path(repo_path);

    let mut registry = HookRegistry::load(repo_path)?;
    let mut outcomes = Vec::new();

    let hooks_to_remove: Vec<String> = if let Some(hook) = specific_hook {
        vec![hook.to_string()]
    } else {
        registry.installed.keys().cloned().collect()
    };

    for hook_name in hooks_to_remove {
        if let Some(entry) = registry.remove(&hook_name) {
            let hook_path = hooks_dir.join(&hook_name);

            // Remove jig wrapper
            if hook_path.exists() {
                std::fs::remove_file(&hook_path)?;
            }

            // Try restoring from .user first (it's the renamed original)
            let user_path = hooks_dir.join(format!("{}.user", hook_name));
            if user_path.exists() {
                std::fs::rename(&user_path, &hook_path)?;
                outcomes.push(UninstallOutcome::RestoredUser(hook_name.clone()));

                // Clean up backup if it exists
                if let Some(backup_name) = &entry.backed_up_to {
                    let backup_path = hooks_dir.join(backup_name);
                    if backup_path.exists() {
                        let _ = std::fs::remove_file(&backup_path);
                    }
                }
                continue;
            }

            // Try restoring from backup
            if let Some(backup_name) = &entry.backed_up_to {
                let backup_path = hooks_dir.join(backup_name);
                if backup_path.exists() {
                    std::fs::copy(&backup_path, &hook_path)?;
                    let _ = std::fs::remove_file(&backup_path);
                    outcomes.push(UninstallOutcome::RestoredBackup {
                        hook: hook_name,
                        backup: backup_name.clone(),
                    });
                    continue;
                }
            }

            outcomes.push(UninstallOutcome::Removed(hook_name));
        }
    }

    // If all hooks removed, delete the registry file
    if specific_hook.is_none() && registry.installed.is_empty() {
        if registry_path.exists() {
            std::fs::remove_file(&registry_path)?;
        }
    } else {
        registry.save(repo_path)?;
    }

    Ok(UninstallResult { outcomes })
}
