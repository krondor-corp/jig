//! Git repository handle — thin wrapper around `git2::Repository`.

use std::path::{Path, PathBuf};

use super::branch::Branch;
use super::commit::Oid;
use super::diff::Diff;
use super::error::{GitError, Result};

/// Wrapper around `git2::Repository` providing jig-specific git operations.
pub struct Repo {
    inner: git2::Repository,
}

impl Repo {
    /// Open a repository by discovering from the current directory.
    pub fn discover() -> Result<Self> {
        let inner = git2::Repository::discover(".").map_err(|_| GitError::NotInRepo)?;
        Ok(Self::wrap(inner))
    }

    /// Open a repository at a specific path.
    pub fn open(path: &Path) -> Result<Self> {
        let inner = git2::Repository::open(path)?;
        Ok(Self::wrap(inner))
    }

    fn wrap(inner: git2::Repository) -> Self {
        let repo = Self { inner };
        repo.ensure_shallow_marker();
        repo
    }

    /// Workaround for a libgit2 quirk where internal stat of `.git/shallow`
    /// surfaces ENOENT as a fatal error during operations like worktree
    /// iteration on macOS (observed against libgit2 1.9.2). An empty
    /// `shallow` file is git's documented "no shallow refs" state, so
    /// creating it is a no-op for git's own behavior.
    fn ensure_shallow_marker(&self) {
        let shallow = self.inner.commondir().join("shallow");
        if !shallow.exists() {
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(&shallow);
        }
    }

    /// Access the underlying git2::Repository.
    pub fn inner(&self) -> &git2::Repository {
        &self.inner
    }

    // ------------------------------------------------------------------
    // Repository info
    // ------------------------------------------------------------------

    /// Get the root directory (workdir) of the repository.
    pub fn root(&self) -> Result<PathBuf> {
        self.inner
            .workdir()
            .map(|p| p.to_path_buf())
            .ok_or(GitError::NotInRepo)
    }

    /// Get the common git directory (`.git/` for clones, `.git/worktrees/<name>/` for linked worktrees).
    pub fn common_dir(&self) -> PathBuf {
        self.inner.commondir().to_path_buf()
    }

    /// Path to the original clone's working directory.
    pub fn clone_path(&self) -> PathBuf {
        self.inner
            .commondir()
            .parent()
            .expect("git common dir must have a parent")
            .to_path_buf()
    }

    /// Whether this repo is the original clone (not a linked worktree).
    pub fn is_clone(&self) -> bool {
        !self.inner.is_worktree()
    }

    /// Open the original clone repo. Re-opens at root if already the clone.
    pub fn open_clone(&self) -> Result<Self> {
        if self.is_clone() {
            Self::open(&self.root()?)
        } else {
            Self::open(&self.clone_path())
        }
    }

    /// Jig worktrees directory (`.jig/` under clone root).
    pub fn worktrees_path(&self) -> PathBuf {
        self.clone_path().join(super::WORKTREES_DIR)
    }

    // ------------------------------------------------------------------
    // Remote operations
    // ------------------------------------------------------------------

    /// Fetch from a remote. If `refspecs` is empty, fetches all refs.
    pub fn fetch(&self, remote: &str, refspecs: &[&str]) -> Result<()> {
        let mut remote = self.inner.find_remote(remote)?;
        remote.fetch(refspecs, None, None)?;
        Ok(())
    }

    /// Returns `true` if a remote with the given name is configured.
    pub fn has_remote(&self, name: &str) -> bool {
        self.inner.find_remote(name).is_ok()
    }

    // ------------------------------------------------------------------
    // Branch operations
    // ------------------------------------------------------------------

    /// Check if a branch exists (local or remote).
    pub fn remote_branch_exists(&self, branch: &Branch) -> bool {
        let name: &str = branch;
        let remote_ref = if name.starts_with("origin/") {
            name.to_string()
        } else {
            format!("origin/{}", name)
        };
        self.inner
            .find_branch(&remote_ref, git2::BranchType::Remote)
            .is_ok()
    }

    pub fn branch_exists(&self, branch: &Branch) -> Result<bool> {
        let name: &str = branch;
        let local = name.strip_prefix("origin/").unwrap_or(name);

        if self
            .inner
            .find_branch(local, git2::BranchType::Local)
            .is_ok()
        {
            return Ok(true);
        }
        if self
            .inner
            .find_branch(&format!("origin/{}", local), git2::BranchType::Remote)
            .is_ok()
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Get the HEAD commit OID.
    pub fn head_oid(&self) -> Result<Oid> {
        let head = self.inner.head()?;
        head.target()
            .map(Oid::new)
            .ok_or_else(|| GitError::BranchNotFound("HEAD".to_string()))
    }

    /// Get the current branch name.
    pub fn current_branch(&self) -> Result<Branch> {
        let head = self.inner.head()?;
        head.shorthand()
            .map(Branch::new)
            .ok_or_else(|| GitError::BranchNotFound("HEAD".to_string()))
    }

    /// Get the base branch (what to diff/rebase against) for the current HEAD.
    ///
    /// jig stores this in the dedicated `branch.<name>.jigBase` config key.
    /// Falls back to git's upstream tracking ref for worktrees created before
    /// that key existed.
    pub fn base_branch(&self) -> Result<Branch> {
        let head = self.inner.head()?;
        let branch_name = head
            .shorthand()
            .ok_or_else(|| GitError::BranchNotFound("HEAD".to_string()))?;

        // Preferred: jig's own base-branch key.
        if let Ok(config) = self.inner.config() {
            if let Ok(base) = config.get_string(&format!("branch.{branch_name}.jigBase")) {
                if !base.is_empty() {
                    return Ok(Branch::new(base));
                }
            }
        }

        // Legacy fallback: git's upstream tracking ref.
        let local = self
            .inner
            .find_branch(branch_name, git2::BranchType::Local)?;
        let upstream = local
            .upstream()
            .map_err(|_| GitError::BranchNotFound(format!("upstream of {}", branch_name)))?;
        let name = upstream
            .name()?
            .ok_or_else(|| GitError::BranchNotFound("upstream".to_string()))?;
        Ok(Branch::new(name))
    }

    // ------------------------------------------------------------------
    // Worktree operations
    // ------------------------------------------------------------------

    /// List all linked worktrees (not the main clone).
    pub fn list_worktrees(&self) -> Result<Vec<super::Worktree>> {
        let wt_names = self.inner.worktrees()?;
        let mut worktrees = Vec::with_capacity(wt_names.len());
        for i in 0..wt_names.len() {
            let Some(name) = wt_names.get(i) else {
                continue;
            };
            let Ok(wt) = self.inner.find_worktree(name) else {
                continue;
            };
            let Ok(worktree) = super::Worktree::open(wt.path()) else {
                continue;
            };
            worktrees.push(worktree);
        }
        Ok(worktrees)
    }

    /// Create a worktree for `branch`, forking from `base` if the branch
    /// doesn't exist yet. Ensures the base branch is also checked out.
    pub fn create_worktree(&self, branch: &Branch, base: &Branch) -> Result<PathBuf> {
        self.prune_stale_worktrees();

        if self.is_branch_checked_out(branch)? {
            return Err(GitError::WorktreeExists(branch.to_string()));
        }

        let branch_str: &str = branch;
        let path = self.worktrees_path().join(branch_str);
        self.add_worktree(&path, branch, base)?;

        Ok(path)
    }

    /// Remove this repo's worktree registration. Errors if not a linked worktree.
    pub fn remove(&self, force: bool) -> Result<()> {
        if !self.inner.is_worktree() {
            return Err(GitError::NotInWorktree);
        }
        if !force && self.has_uncommitted_changes()? {
            return Err(GitError::UncommittedChanges);
        }

        let branch = self.current_branch()?;
        let clone = self.open_clone()?;
        clone.prune_worktree(&branch, force)?;
        Ok(())
    }

    /// Prune stale (invalid) worktree registrations.
    pub fn prune_stale_worktrees(&self) {
        self.for_each_worktree(|_name, wt| {
            if wt.validate().is_err() {
                let mut opts = git2::WorktreePruneOptions::new();
                let _ = wt.prune(Some(&mut opts));
            }
            Ok(())
        })
        .ok();
    }

    /// Prune a worktree by its branch name.
    pub fn prune_worktree(&self, branch: &Branch, force: bool) -> Result<()> {
        let name: &str = branch;
        let local = name.strip_prefix("origin/").unwrap_or(name);
        let wt_name = local.replace('/', "-");
        self.prune_worktree_named(&wt_name, force)
    }

    // ------------------------------------------------------------------
    // Status & diff
    // ------------------------------------------------------------------

    /// Check for uncommitted changes.
    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let statuses = self.inner.statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))?;
        Ok(!statuses.is_empty())
    }

    /// Get commits ahead of a base branch.
    pub fn commits_ahead(&self, base: &Branch) -> Result<Vec<String>> {
        let base_str: &str = base;
        let base_oid = match self.inner.revparse_single(base_str) {
            Ok(obj) => match obj.peel(git2::ObjectType::Commit) {
                Ok(c) => c.id(),
                Err(_) => return Ok(Vec::new()),
            },
            Err(_) => return Ok(Vec::new()),
        };

        let head_oid = match self.head_oid() {
            Ok(oid) => oid,
            Err(_) => return Ok(Vec::new()),
        };

        let mut revwalk = self.inner.revwalk()?;
        revwalk.push(head_oid.inner())?;
        revwalk.hide(base_oid)?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

        let mut commits = Vec::new();
        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = self.inner.find_commit(oid)?;
            let short_id = &oid.to_string()[..7];
            let summary = commit.summary().unwrap_or("");
            commits.push(format!("{} {}", short_id, summary));
        }

        Ok(commits)
    }

    /// Get the diff against a base branch.
    pub fn diff(&self, base: &Branch) -> Result<Diff<'_>> {
        let base_str: &str = base;
        let base_tree = self.resolve_to_commit(base_str)?.tree()?;
        let head_tree = self
            .inner
            .head()?
            .peel(git2::ObjectType::Commit)?
            .into_commit()
            .map_err(|_| git2::Error::from_str("HEAD is not a commit"))?
            .tree()?;

        let raw = self
            .inner
            .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?;
        Ok(Diff::new(raw))
    }

    // ------------------------------------------------------------------
    // Merge
    // ------------------------------------------------------------------

    /// Merge a branch into the current HEAD.
    pub fn merge_branch(&self, branch: &Branch) -> Result<()> {
        let name: &str = branch;
        let local = name.strip_prefix("origin/").unwrap_or(name);

        let branch_ref = self
            .inner
            .find_branch(local, git2::BranchType::Local)
            .or_else(|_| {
                self.inner
                    .find_branch(&format!("origin/{}", local), git2::BranchType::Remote)
            })
            .map_err(|_| GitError::BranchNotFound(branch.to_string()))?;

        let annotated = self
            .inner
            .reference_to_annotated_commit(&branch_ref.into_reference())?;
        let (analysis, _) = self.inner.merge_analysis(&[&annotated])?;

        if analysis.is_up_to_date() {
            return Ok(());
        }

        if analysis.is_fast_forward() {
            let target_oid = annotated.id();
            let mut reference = self.inner.head()?;
            reference.set_target(target_oid, &format!("merge {}: Fast-forward", branch))?;
            self.inner
                .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
            return Ok(());
        }

        if analysis.is_normal() {
            self.inner.merge(&[&annotated], None, None)?;

            let mut index = self.inner.index()?;
            if index.has_conflicts() {
                self.inner.cleanup_state()?;
                return Err(GitError::MergeConflict(branch.to_string()));
            }

            let tree_oid = index.write_tree()?;
            let tree = self.inner.find_tree(tree_oid)?;
            let head_commit = self
                .inner
                .head()?
                .peel(git2::ObjectType::Commit)?
                .into_commit()
                .map_err(|_| git2::Error::from_str("HEAD is not a commit"))?;
            let merge_commit = self.inner.find_commit(annotated.id())?;
            let sig = self
                .inner
                .signature()
                .or_else(|_| git2::Signature::now("jig", "jig@localhost"))?;

            self.inner.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("Merge branch '{}'", branch),
                &tree,
                &[&head_commit, &merge_commit],
            )?;
            self.inner.cleanup_state()?;
            return Ok(());
        }

        Err(GitError::MergeConflict(branch.to_string()))
    }

    // ------------------------------------------------------------------
    // Remote operations
    // ------------------------------------------------------------------

    /// Fast-forward the current branch to match its remote tracking ref.
    pub fn fast_forward(&self) -> Result<bool> {
        let branch = self.current_branch()?;
        self.fast_forward_branch(&branch, true)
    }

    /// Fast-forward a local branch to match its remote tracking ref.
    ///
    /// When `checkout` is true, also updates the working tree (use when the
    /// branch is currently checked out). When false, only the ref is moved.
    /// Creates the local branch if it doesn't exist yet.
    ///
    /// Returns `true` if the ref was advanced, `false` if already up to date.
    pub fn fast_forward_branch(&self, branch: &Branch, checkout: bool) -> Result<bool> {
        let name: &str = branch;
        let local = name.strip_prefix("origin/").unwrap_or(name);
        let remote_ref_name = format!("origin/{}", local);

        let remote_branch = self
            .inner
            .find_branch(&remote_ref_name, git2::BranchType::Remote)
            .map_err(|_| GitError::BranchNotFound(remote_ref_name.clone()))?;

        let remote_oid = remote_branch
            .get()
            .target()
            .ok_or_else(|| GitError::BranchNotFound(remote_ref_name.clone()))?;

        let local_ref_name = format!("refs/heads/{}", local);

        let local_oid = match self.inner.find_reference(&local_ref_name) {
            Ok(r) => match r.target() {
                Some(oid) => oid,
                None => return Err(GitError::BranchNotFound(local.to_string())),
            },
            Err(_) => {
                self.inner.reference(
                    &local_ref_name,
                    remote_oid,
                    false,
                    &format!("create {} from {}", local, remote_ref_name),
                )?;
                if checkout {
                    self.inner
                        .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
                }
                return Ok(true);
            }
        };

        if local_oid == remote_oid {
            return Ok(false);
        }

        if !self.inner.graph_descendant_of(remote_oid, local_oid)? {
            return Err(GitError::MergeConflict(remote_ref_name));
        }

        self.inner.reference(
            &local_ref_name,
            remote_oid,
            true,
            &format!("fast-forward {} to {}", local, remote_ref_name),
        )?;

        if checkout {
            self.inner
                .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        }

        Ok(true)
    }

    /// Push a branch to origin.
    pub fn push_branch(&self, branch: &Branch) -> Result<()> {
        let name: &str = branch;
        let local = name.strip_prefix("origin/").unwrap_or(name);
        let refspec = format!("refs/heads/{local}:refs/heads/{local}");

        let mut remote = self
            .inner
            .find_remote("origin")
            .map_err(|e| GitError::PushFailed(format!("no remote 'origin': {e}")))?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            } else if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                git2::Cred::credential_helper(
                    &git2::Config::open_default()?,
                    url,
                    username_from_url,
                )
            } else if allowed_types.contains(git2::CredentialType::DEFAULT) {
                git2::Cred::default()
            } else {
                Err(git2::Error::from_str("no available credentials"))
            }
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut push_opts))
            .map_err(|e| GitError::PushFailed(format!("push origin {local} failed: {e}")))?;

        Ok(())
    }

    /// Create a local branch from `origin/{base}` and push it to origin.
    ///
    /// Tolerates a pre-existing local branch (pushes the existing one).
    pub fn create_and_push_branch(&self, branch: &Branch, base: &Branch) -> Result<()> {
        let base_ref: &str = base;
        let base_local = base_ref.strip_prefix("origin/").unwrap_or(base_ref);
        let remote_ref = format!("origin/{}", base_local);

        let reference = self
            .inner
            .find_branch(&remote_ref, git2::BranchType::Remote)
            .map_err(|e| GitError::BranchNotFound(format!("{}: {}", remote_ref, e)))?;
        let commit = reference.get().peel_to_commit()?;

        let branch_str: &str = branch;
        match self.inner.branch(branch_str, &commit, false) {
            Ok(_) => {}
            Err(e) if e.code() == git2::ErrorCode::Exists => {}
            Err(e) => return Err(e.into()),
        }

        self.push_branch(branch)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Check if a branch is checked out in the main workdir or any linked worktree.
    fn is_branch_checked_out(&self, branch: &Branch) -> Result<bool> {
        let clone = self.open_clone()?;
        let name: &str = branch;
        let local = name.strip_prefix("origin/").unwrap_or(name);

        if let Ok(b) = clone.current_branch() {
            if &*b == local {
                return Ok(true);
            }
        }

        Ok(clone
            .list_worktrees()?
            .iter()
            .any(|wt| wt.branch().is_ok_and(|b| &*b == local)))
    }

    /// Low-level: create a git2 worktree at `path` for `branch`, forking
    /// from `base` if the branch doesn't exist yet. Creates parent dirs.
    ///
    /// Resolution order for the starting commit:
    /// 1. Local branch `<branch>` exists → use it as-is.
    /// 2. Remote branch `origin/<branch>` exists → create local from it and
    ///    set upstream to `origin/<branch>`.
    /// 3. Neither → fork from `base` (errors if `base` can't be resolved).
    fn add_worktree(&self, path: &Path, branch: &Branch, base: &Branch) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let branch_str: &str = branch;
        let local = branch_str.strip_prefix("origin/").unwrap_or(branch_str);
        // git stores worktree metadata in .git/worktrees/<name>/ — slashes
        // in the name create nested dirs that don't exist. Use a flat name.
        let wt_name = local.replace('/', "-");

        // The base branch jig diffs/rebases against, stored separately from
        // git's upstream tracking (see the jigBase note below).
        let jig_base: String;

        if let Ok(branch_ref) = self.inner.find_branch(local, git2::BranchType::Local) {
            // Case 1: branch already exists locally.
            let reference = branch_ref.into_reference();
            let mut opts = git2::WorktreeAddOptions::new();
            opts.reference(Some(&reference));
            self.inner.worktree(&wt_name, path, Some(&opts))?;
            let base_str: &str = base;
            jig_base = base_str
                .strip_prefix("origin/")
                .unwrap_or(base_str)
                .to_string();
        } else if let Ok(remote_commit) = self.resolve_to_commit(&format!("origin/{}", local)) {
            // Case 2: branch exists on origin — check it out and track it.
            // The configured base is intentionally ignored here; the remote
            // branch is the authoritative starting point.
            tracing::info!(
                branch = local,
                base = %base,
                "branch found on origin; ignoring configured base"
            );
            let new_branch = self.inner.branch(local, &remote_commit, false)?;
            let reference = new_branch.into_reference();
            let mut opts = git2::WorktreeAddOptions::new();
            opts.reference(Some(&reference));
            self.inner.worktree(&wt_name, path, Some(&opts))?;
            jig_base = format!("origin/{}", local);
        } else {
            // Case 3: branch is new — fork from base.
            let base_str: &str = base;
            let start_commit = self.find_valid_start_point(base_str)?;
            let new_branch = self.inner.branch(local, &start_commit, false)?;
            let reference = new_branch.into_reference();
            let mut opts = git2::WorktreeAddOptions::new();
            opts.reference(Some(&reference));
            self.inner.worktree(&wt_name, path, Some(&opts))?;
            jig_base = base_str
                .strip_prefix("origin/")
                .unwrap_or(base_str)
                .to_string();
        }

        let wt_repo = Self::open(path)?;

        // jig tracks the *base* branch (what to diff/rebase against) in its
        // own config key rather than overloading git's upstream tracking.
        // Overloading upstream breaks a plain `git push` under
        // push.default=simple: when the base name (e.g. `dev`) differs from
        // the branch name (e.g. `feature/x`), git refuses with "upstream
        // branch ... does not match the name of your current branch". The
        // base is read back by `base_branch()`.
        if let Ok(mut config) = wt_repo.inner.config() {
            let _ = config.set_str(&format!("branch.{local}.jigBase"), &jig_base);
            // Let `git push` create and track origin/<branch> on first push.
            let _ = config.set_bool("push.autoSetupRemote", true);
        }

        // Point git's real upstream at the branch's own remote-tracking ref
        // when it already exists (so `git pull`/`git push` behave normally);
        // otherwise leave it unset and let push.autoSetupRemote configure it
        // on the first push.
        if self.resolve_to_commit(&format!("origin/{local}")).is_ok() {
            if let Ok(mut local_branch) = wt_repo.inner.find_branch(local, git2::BranchType::Local)
            {
                let _ = local_branch.set_upstream(Some(&format!("origin/{local}")));
            }
        }

        Ok(())
    }

    fn for_each_worktree(
        &self,
        mut f: impl FnMut(&str, git2::Worktree) -> Result<()>,
    ) -> Result<()> {
        let wt_names = self.inner.worktrees()?;
        for i in 0..wt_names.len() {
            let Some(name) = wt_names.get(i) else {
                continue;
            };
            let Ok(wt) = self.inner.find_worktree(name) else {
                continue;
            };
            f(name, wt)?;
        }
        Ok(())
    }

    fn prune_worktree_named(&self, name: &str, force: bool) -> Result<()> {
        let wt = self.inner.find_worktree(name)?;
        let mut opts = git2::WorktreePruneOptions::new();
        opts.valid(true);
        opts.working_tree(true);
        if force {
            opts.locked(true);
        }
        wt.prune(Some(&mut opts))?;
        Ok(())
    }

    fn resolve_to_commit(&self, spec: &str) -> Result<git2::Commit<'_>> {
        let obj = self
            .inner
            .revparse_single(spec)
            .map_err(|_| GitError::BranchNotFound(spec.to_string()))?;
        Ok(obj
            .peel(git2::ObjectType::Commit)?
            .into_commit()
            .map_err(|_| git2::Error::from_str("not a commit"))?)
    }

    fn find_valid_start_point(&self, base_branch: &str) -> Result<git2::Commit<'_>> {
        if let Ok(commit) = self.resolve_to_commit(base_branch) {
            return Ok(commit);
        }

        if !base_branch.starts_with("origin/") {
            if let Ok(commit) = self.resolve_to_commit(&format!("origin/{}", base_branch)) {
                return Ok(commit);
            }
        }

        Err(GitError::BranchNotFound(base_branch.to_string()))
    }
}

#[cfg(test)]
mod remote_tests {
    use super::*;
    use tempfile::TempDir;

    fn init_repo_with_commit(dir: &Path) -> git2::Repository {
        let repo = git2::Repository::init(dir).unwrap();
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_bool("commit.gpgsign", false).unwrap();
        }
        {
            let mut index = repo.index().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
                .unwrap();
        }
        repo.head()
            .unwrap()
            .rename("refs/heads/main", true, "init main")
            .unwrap();
        repo
    }

    #[test]
    fn has_remote_true_when_configured() {
        let tmp = TempDir::new().unwrap();
        let git = init_repo_with_commit(tmp.path());
        let path = tmp.path().to_str().unwrap();
        git.remote("origin", path).unwrap();

        let repo = Repo::open(tmp.path()).unwrap();
        assert!(repo.has_remote("origin"));
    }

    #[test]
    fn has_remote_false_when_not_configured() {
        let tmp = TempDir::new().unwrap();
        let _ = init_repo_with_commit(tmp.path());

        let repo = Repo::open(tmp.path()).unwrap();
        assert!(!repo.has_remote("origin"));
        assert!(!repo.has_remote("upstream"));
    }

    #[test]
    fn find_valid_start_point_no_head_fallback() {
        let tmp = TempDir::new().unwrap();
        let _ = init_repo_with_commit(tmp.path());

        let repo = Repo::open(tmp.path()).unwrap();
        // "origin/main" doesn't exist, no remote configured, no HEAD fallback
        let result = repo.find_valid_start_point("origin/main");
        assert!(
            matches!(result, Err(GitError::BranchNotFound(ref s)) if s == "origin/main"),
            "expected BranchNotFound, got {:?}",
            result
        );
    }

    #[test]
    fn find_valid_start_point_resolves_local_branch() {
        let tmp = TempDir::new().unwrap();
        let _ = init_repo_with_commit(tmp.path());

        let repo = Repo::open(tmp.path()).unwrap();
        let result = repo.find_valid_start_point("main");
        assert!(result.is_ok(), "expected Ok for existing local branch");
    }

    #[test]
    fn create_worktree_uses_remote_branch_when_exists() {
        let tmp = TempDir::new().unwrap();
        let git = init_repo_with_commit(tmp.path());

        // Create feat/xyz locally, then expose it via self-remote
        let head = git.head().unwrap().peel_to_commit().unwrap();
        git.branch("feat/xyz", &head, false).unwrap();
        let remote_url = tmp.path().to_str().unwrap();
        git.remote("origin", remote_url).unwrap();
        git.find_remote("origin")
            .unwrap()
            .fetch(&[] as &[&str], None, None)
            .unwrap();

        // Delete local feat/xyz so only origin/feat/xyz remains
        git.find_branch("feat/xyz", git2::BranchType::Local)
            .unwrap()
            .delete()
            .unwrap();

        let repo = Repo::open(tmp.path()).unwrap();
        let branch: Branch = "feat/xyz".into();
        let base: Branch = "origin/main".into();
        let path = repo.create_worktree(&branch, &base).unwrap();

        // The worktree should exist and be on feat/xyz
        let wt_repo = Repo::open(&path).unwrap();
        let current = wt_repo.current_branch().unwrap();
        assert_eq!(&*current, "feat/xyz");

        // The local branch should be at the same commit as origin/feat/xyz
        let local_oid = wt_repo
            .inner()
            .find_branch("feat/xyz", git2::BranchType::Local)
            .unwrap()
            .get()
            .target()
            .unwrap();
        let remote_oid = repo
            .inner()
            .find_branch("origin/feat/xyz", git2::BranchType::Remote)
            .unwrap()
            .get()
            .target()
            .unwrap();
        assert_eq!(
            local_oid, remote_oid,
            "local branch should track origin/feat/xyz"
        );
    }

    /// When the branch was created by jig (local + origin exist, e.g. via
    /// `create_and_push_branch`), `create_worktree` must set
    /// `push.autoSetupRemote = true` and track `origin/<branch>` — not the
    /// configured base — so that `git push` works from the new worktree.
    #[test]
    fn create_worktree_case1_local_and_remote_sets_auto_push_and_upstream() {
        let tmp = TempDir::new().unwrap();
        let git = init_repo_with_commit(tmp.path());

        // Simulate create_and_push_branch: local branch + push to self-remote.
        let head = git.head().unwrap().peel_to_commit().unwrap();
        git.branch("feature/integration", &head, false).unwrap();
        let remote_url = tmp.path().to_str().unwrap();
        git.remote("origin", remote_url).unwrap();
        git.find_remote("origin")
            .unwrap()
            .fetch(&[] as &[&str], None, None)
            .unwrap();

        // Both local and origin/feature/integration now exist — Case 1.
        let repo = Repo::open(tmp.path()).unwrap();
        let branch: Branch = "feature/integration".into();
        let base: Branch = "origin/main".into();
        let path = repo.create_worktree(&branch, &base).unwrap();

        let wt_repo = Repo::open(&path).unwrap();

        // push.autoSetupRemote must be set so raw `git push` works.
        let config = wt_repo.inner().config().unwrap();
        assert!(
            config.get_bool("push.autoSetupRemote").unwrap_or(false),
            "push.autoSetupRemote must be true in Case 1 worktree"
        );

        // Upstream must point at origin/feature/integration, not the base branch.
        let local_branch = wt_repo
            .inner()
            .find_branch("feature/integration", git2::BranchType::Local)
            .unwrap();
        let upstream = local_branch
            .upstream()
            .expect("upstream must be configured");
        let upstream_name = upstream.name().unwrap().unwrap();
        assert_eq!(
            upstream_name, "origin/feature/integration",
            "upstream should track origin/feature/integration, not the base branch"
        );
    }

    #[test]
    fn new_branch_does_not_set_base_as_git_upstream() {
        let tmp = TempDir::new().unwrap();
        let _ = init_repo_with_commit(tmp.path());

        let repo = Repo::open(tmp.path()).unwrap();
        let branch: Branch = "feature/x".into();
        let base: Branch = "main".into();
        let path = repo.create_worktree(&branch, &base).unwrap();

        let wt_repo = Repo::open(&path).unwrap();

        // The base must NOT be wired into git's upstream tracking — doing so
        // breaks `git push` under push.default=simple (the original bug).
        let local = wt_repo
            .inner()
            .find_branch("feature/x", git2::BranchType::Local)
            .unwrap();
        assert!(
            local.upstream().is_err(),
            "new branch must not have a git upstream pointing at the base"
        );

        // But jig must still resolve the base for diff/rebase purposes.
        assert_eq!(&*wt_repo.base_branch().unwrap(), "main");

        // And first-push must auto-create origin/<branch>.
        let auto = wt_repo
            .inner()
            .config()
            .unwrap()
            .get_bool("push.autoSetupRemote")
            .unwrap();
        assert!(auto, "push.autoSetupRemote should be enabled");
    }
}

#[cfg(test)]
mod shallow_tests {
    use super::*;
    use tempfile::TempDir;

    fn init_test_repo(dir: &Path) -> git2::Repository {
        let repo = git2::Repository::init(dir).unwrap();
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_bool("commit.gpgsign", false).unwrap();
        }
        {
            let mut index = repo.index().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
                .unwrap();
        }
        repo
    }

    #[test]
    fn open_creates_shallow_marker_when_missing() {
        let tmp = TempDir::new().unwrap();
        let _ = init_test_repo(tmp.path());
        let shallow = tmp.path().join(".git").join("shallow");
        assert!(!shallow.exists(), "fresh repo should have no shallow file");

        let _repo = Repo::open(tmp.path()).unwrap();
        assert!(
            shallow.exists(),
            "Repo::open must create .git/shallow as a libgit2 quirk workaround"
        );
        assert_eq!(
            std::fs::metadata(&shallow).unwrap().len(),
            0,
            "shallow marker must be empty (= no shallow refs)"
        );
    }

    #[test]
    fn open_preserves_existing_shallow_file() {
        let tmp = TempDir::new().unwrap();
        let repo = init_test_repo(tmp.path());

        let head_oid = repo.head().unwrap().target().unwrap().to_string();
        let shallow_contents = format!("{}\n", head_oid);
        drop(repo);

        let shallow = tmp.path().join(".git").join("shallow");
        std::fs::write(&shallow, shallow_contents.as_bytes()).unwrap();

        let _repo = Repo::open(tmp.path()).unwrap();
        let contents = std::fs::read(&shallow).unwrap();
        assert_eq!(
            contents,
            shallow_contents.as_bytes(),
            "existing shallow contents must not be clobbered"
        );
    }
}
