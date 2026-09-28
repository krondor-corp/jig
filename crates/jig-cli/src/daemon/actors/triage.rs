//! Triage actor — polls for triageable issues and runs ephemeral triage agents
//! as direct subprocesses. Owns the TriageTracker for dedup and stuck detection.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::context::{self, RepoCtx};
use jig_core::agents;
use jig_core::exec::{Exec, Timeout};
use jig_core::git::Repo;
use jig_core::issues::issue::{IssueFilter, IssueStatus};
use jig_core::issues::Issue;

use super::Actor;
use crate::daemon::TickContext;

pub struct TriageRequest {
    pub ctx: TickContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageEntry {
    pub spawned_at: i64,
    pub issue_id: String,
    pub repo_name: String,
    /// Agent model this triage runs under, for display.
    pub model: String,
}

#[derive(Default)]
pub struct TriageActor {
    tracker: Mutex<HashMap<String, TriageEntry>>,
}

impl TriageActor {
    pub fn is_active(&self, issue_id: &str) -> bool {
        self.tracker.lock().unwrap().contains_key(issue_id)
    }

    pub fn active_entries(&self) -> Vec<TriageEntry> {
        self.tracker.lock().unwrap().values().cloned().collect()
    }

    pub fn remove(&self, issue_id: &str) {
        self.tracker.lock().unwrap().remove(issue_id);
    }

    fn register(&self, issue_id: String, entry: TriageEntry) {
        self.tracker.lock().unwrap().insert(issue_id, entry);
    }
}

struct TriageIssue {
    repo_root: PathBuf,
    issue: Issue,
    /// Hard wall-clock limit for the subprocess, from `triage.timeout_seconds`.
    timeout_seconds: i64,
}

impl Actor for TriageActor {
    type Request = TriageRequest;
    type Response = ();

    const NAME: &'static str = "jig-triage";
    const QUEUE_SIZE: usize = 1;

    fn handle(&self, req: TriageRequest) {
        let now = chrono::Utc::now().timestamp();

        // Triage timeouts are enforced directly on the subprocess (see
        // run_triage_subprocess); the tracker exists for dedup and for the
        // `jig ps` display, so there is no stuck-entry sweep to do here.

        // Poll for new triageable issues
        let global = &req.ctx.config;
        for entry in req.ctx.repos.iter() {
            let repo_root = entry.path.clone();
            let repo_name = repo_root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Validity guard only; triage needs no worktree or branch data.
            if let Err(e) = Repo::open(&repo_root) {
                tracing::debug!(repo = %repo_name, error = %e, "failed to open repo");
                continue;
            }

            let cfg = match RepoCtx::from_path(&repo_root) {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::debug!(repo = %repo_name, error = %e, "failed to load config");
                    continue;
                }
            };

            if !cfg.config.triage.enabled {
                continue;
            }

            let provider = match cfg.issue_provider(global) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!(repo = %repo_name, error = %e, "failed to create issue provider");
                    continue;
                }
            };

            // Triage runs as a subprocess with no worktree, branch or mux
            // window, so it deliberately does not consume the worker budget.
            // Gating it on max_concurrent_workers silently stopped triage
            // whenever the workers were busy.

            let triageable = match provider.list(&IssueFilter {
                status: Some(IssueStatus::Triage),
                ..Default::default()
            }) {
                Ok(issues) => issues,
                Err(_) => continue,
            };

            let timeout_seconds = cfg.config.triage.timeout_seconds;
            let model = cfg
                .config
                .triage
                .model
                .clone()
                .unwrap_or_else(|| "default".to_string());

            for issue in triageable.into_iter() {
                if self.is_active(issue.id()) {
                    continue;
                }

                self.register(
                    issue.id().to_string(),
                    TriageEntry {
                        spawned_at: now,
                        issue_id: issue.id().to_string(),
                        repo_name: repo_name.clone(),
                        model: model.clone(),
                    },
                );

                let issue_id = issue.id().to_string();
                let ti = TriageIssue {
                    repo_root: repo_root.clone(),
                    issue,
                    timeout_seconds,
                };

                run_single(&ti);

                // The subprocess is synchronous, so clear the tracker here to
                // keep is_active() and the ps display honest.
                self.remove(&issue_id);
            }
        }
    }
}

fn run_single(issue: &TriageIssue) {
    tracing::info!(
        issue = %issue.issue.id(),
        timeout_seconds = issue.timeout_seconds,
        "running triage subprocess"
    );

    match run_triage_subprocess(&issue.repo_root, &issue.issue, issue.timeout_seconds) {
        Ok(()) => {
            tracing::info!(
                issue = %issue.issue.id(),
                "triage subprocess completed successfully"
            );
        }
        Err(msg) => {
            tracing::warn!(
                issue = %issue.issue.id(),
                "triage subprocess failed: {}", msg
            );
        }
    }
}

pub(crate) fn run_triage_subprocess(
    repo_root: &Path,
    issue: &Issue,
    timeout_seconds: i64,
) -> std::result::Result<(), String> {
    let prompt = crate::prompts::triage::triage_prompt(issue);

    let jig_toml = context::RepoConfig::load(repo_root)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();
    let agent = agents::Agent::from_config(
        &jig_toml.agent.agent_type,
        jig_toml.triage.model.as_deref(),
        &[],
    )
    .unwrap_or_else(|| agents::Agent::from_config("claude", None, &[]).unwrap());

    let argv = agent
        .once(prompt, crate::prompts::triage::ALLOWED_TOOLS)
        .map_err(|e| e.to_string())?;

    let (cmd, args) = argv.split_first().ok_or("empty triage argv")?;

    let mut command = Command::new(cmd);
    command.args(args);

    let output = Exec::command(command)
        .labeled("triage agent")
        .timeout(Timeout::secs(timeout_seconds.max(1) as u64))
        .in_dir(repo_root)
        .run()
        .map_err(|e| e.to_string())?;

    if !output.success() {
        return Err(format!(
            "triage agent failed: {}",
            output.failure().chars().take(500).collect::<String>()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_dedup() {
        let actor = TriageActor::default();
        actor.register(
            "JIG-1".to_string(),
            TriageEntry {
                spawned_at: 1000,
                issue_id: "JIG-1".to_string(),
                repo_name: "repo".to_string(),
                model: "sonnet".to_string(),
            },
        );
        assert!(actor.is_active("JIG-1"));
        assert!(!actor.is_active("JIG-2"));
        actor.remove("JIG-1");
        assert!(!actor.is_active("JIG-1"));
    }

    #[test]
    fn active_entries_returns_all() {
        let actor = TriageActor::default();
        actor.register(
            "JIG-1".to_string(),
            TriageEntry {
                spawned_at: 100,
                issue_id: "JIG-1".to_string(),
                repo_name: "repo".to_string(),
                model: "sonnet".to_string(),
            },
        );
        actor.register(
            "JIG-2".to_string(),
            TriageEntry {
                spawned_at: 200,
                issue_id: "JIG-2".to_string(),
                repo_name: "repo".to_string(),
                model: "sonnet".to_string(),
            },
        );
        assert_eq!(actor.active_entries().len(), 2);
    }
}
