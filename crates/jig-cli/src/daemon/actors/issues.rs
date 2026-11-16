//! Issues actor — keeps each repo's issues that could be spawned (planned
//! and backlog, not triage), so the web view can list them without calling
//! the issue provider itself.
//!
//! Fetched on the poll interval, like sync and spawn, and when someone asks
//! for a refresh ([`super::super::DaemonShared::refresh_issues`]). Reading
//! the board is a lock and a clone: the page's few-second refresh never
//! reaches Linear.
//!
//! Only this machine's repos, through each repo's own issue provider (its
//! `jig.toml` names a Linear profile, whose key lives in this machine's
//! config). Other machines' issues are theirs to list.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use jig_core::git::{Branch, Repo};
use jig_core::issues::issue::{IssueFilter, IssueStatus};
use jig_core::issues::{Issue, IssuePriority};

use super::Actor;
use crate::daemon::TickContext;

/// The statuses worth offering a spawn button for.
const LISTED: [IssueStatus; 2] = [IssueStatus::Planned, IssueStatus::Backlog];

pub struct IssuesRequest {
    pub ctx: TickContext,
}

/// Every repo's listed issues, as last fetched.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IssueBoard {
    pub repos: Vec<RepoIssues>,
    /// When this was fetched, in unix seconds. `None` before the first fetch.
    pub fetched_at: Option<i64>,
}

/// One repo's issues, or why they could not be listed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoIssues {
    /// The repo's directory name: what [`super::super::commands::WorkerKey`]
    /// and a spawn request name it by.
    pub repo: String,
    pub issues: Vec<IssueEntry>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueEntry {
    pub id: String,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub labels: Vec<String>,
    /// The branch a spawn would create.
    pub branch: Branch,
    /// Matches the repo's auto-spawn labels: the daemon spawns it on its
    /// own once it is planned.
    pub auto: bool,
    /// Has sub-issues: those are spawned, not this.
    pub parent: bool,
    /// Its branch already has a worktree here: being worked on.
    pub spawned: bool,
}

#[derive(Default)]
pub struct IssuesActor {
    board: Mutex<IssueBoard>,
}

impl IssuesActor {
    /// The board as last fetched.
    pub fn board(&self) -> IssueBoard {
        self.board.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl Actor for IssuesActor {
    type Request = IssuesRequest;
    type Response = ();

    const NAME: &'static str = "jig-issues";
    const QUEUE_SIZE: usize = 1;

    fn handle(&self, req: IssuesRequest) {
        let global = &req.ctx.app.config;
        let mut repos = Vec::new();

        for entry in req.ctx.repos.iter() {
            let Ok(cfg) = req.ctx.app.repo_at(&entry.path) else {
                continue;
            };
            // A repo with no issue tracker has nothing to list, which is not
            // an error worth showing.
            if cfg.config.issues.linear.is_none() {
                continue;
            }
            let repo = cfg.name();
            let listed = cfg
                .issue_provider(global)
                .map_err(|e| e.to_string())
                .and_then(|provider| {
                    LISTED
                        .iter()
                        .map(|status| {
                            provider.list(&IssueFilter {
                                status: Some(status.clone()),
                                ..Default::default()
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| e.to_string())
                });

            let worktrees: Vec<Branch> = Repo::open(&entry.path)
                .and_then(|r| r.list_worktrees())
                .unwrap_or_default()
                .iter()
                .filter_map(|wt| wt.branch().ok())
                .collect();
            let labels = cfg.config.issues.auto_spawn_labels.clone();

            repos.push(match listed {
                Ok(lists) => RepoIssues {
                    repo,
                    issues: lists
                        .into_iter()
                        .flatten()
                        .map(|issue| entry_for(&issue, labels.as_deref(), &worktrees))
                        .collect(),
                    error: None,
                },
                Err(error) => {
                    tracing::warn!(%repo, "failed to list issues: {}", error);
                    RepoIssues {
                        repo,
                        issues: Vec::new(),
                        error: Some(error),
                    }
                }
            });
        }

        *self.board.lock().unwrap_or_else(|e| e.into_inner()) = IssueBoard {
            repos,
            fetched_at: Some(chrono::Utc::now().timestamp()),
        };
    }
}

/// `auto_labels` is the repo's `auto_spawn_labels`: unset spawns nothing on
/// its own, an empty list every planned issue (the spawn actor's rule).
fn entry_for(issue: &Issue, auto_labels: Option<&[String]>, worktrees: &[Branch]) -> IssueEntry {
    IssueEntry {
        id: issue.id().to_string(),
        title: issue.title().to_string(),
        status: issue.status().clone(),
        priority: issue.priority().clone(),
        labels: issue.labels().to_vec(),
        branch: issue.branch().clone(),
        auto: auto_labels.is_some_and(|labels| labels.is_empty() || issue.auto(labels)),
        parent: !issue.children().is_empty(),
        spawned: worktrees.contains(issue.branch()),
    }
}
