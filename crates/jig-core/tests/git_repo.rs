//! `Repo` against real repositories on disk.

mod common;

use std::path::{Path, PathBuf};

use common::seeded_repo as init_repo;
use jig_core::git::{Branch, GitError, Repo};
use tempfile::TempDir;

#[test]
fn has_remote_true_when_configured() {
    let tmp = TempDir::new().unwrap();
    let git = init_repo(tmp.path());
    let path = tmp.path().to_str().unwrap();
    git.remote("origin", path).unwrap();

    let repo = Repo::open(tmp.path()).unwrap();
    assert!(repo.has_remote("origin"));
}

#[test]
fn has_remote_false_when_not_configured() {
    let tmp = TempDir::new().unwrap();
    let _ = init_repo(tmp.path());

    let repo = Repo::open(tmp.path()).unwrap();
    assert!(!repo.has_remote("origin"));
    assert!(!repo.has_remote("upstream"));
}

fn add_foreign_worktree(git: &git2::Repository, root: &Path) -> PathBuf {
    let head = git.head().unwrap().peel_to_commit().unwrap();
    let branch = git.branch("claude/branch", &head, false).unwrap();
    let path = root.join(".claude/worktrees/indexing-rework");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut opts = git2::WorktreeAddOptions::new();
    opts.reference(Some(branch.get()));
    git.worktree("indexing-rework", &path, Some(&opts)).unwrap();
    path
}

#[test]
fn foreign_worktrees_are_ignored_not_fatal() {
    let tmp = TempDir::new().unwrap();
    let git = init_repo(tmp.path());
    let repo = Repo::open(tmp.path()).unwrap();
    repo.create_worktree(&"feat/mine".into(), &"main".into())
        .unwrap();
    let foreign = add_foreign_worktree(&git, tmp.path());

    // Discovery sees only jig's worktree — this used to panic in
    // `branch_name` ("worktree path must be under worktrees dir").
    let names: Vec<String> = repo
        .list_worktrees()
        .unwrap()
        .iter()
        .map(|wt| wt.branch_name().to_string())
        .collect();
    assert_eq!(names, vec!["feat/mine"]);

    assert!(matches!(
        jig_core::git::Worktree::open(&foreign),
        Err(GitError::NotJigWorktree(_))
    ));
    // git2 reports resolved paths (/private/var vs /var on macOS).
    assert_eq!(
        repo.foreign_worktree_paths().unwrap(),
        vec![foreign.canonicalize().unwrap()]
    );
    assert_eq!(repo.linked_worktree_paths().unwrap().len(), 2);
}

#[test]
fn branch_in_foreign_worktree_still_counts_as_checked_out() {
    let tmp = TempDir::new().unwrap();
    let git = init_repo(tmp.path());
    add_foreign_worktree(&git, tmp.path());

    let repo = Repo::open(tmp.path()).unwrap();
    assert!(matches!(
        repo.create_worktree(&"claude/branch".into(), &"main".into()),
        Err(GitError::WorktreeExists(_))
    ));
}

#[test]
fn create_worktree_uses_remote_branch_when_exists() {
    let tmp = TempDir::new().unwrap();
    let git = init_repo(tmp.path());

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

#[test]
fn create_worktree_case1_local_and_remote_sets_auto_push_and_upstream() {
    let tmp = TempDir::new().unwrap();
    let git = init_repo(tmp.path());

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
    let _ = init_repo(tmp.path());

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

#[test]
fn open_creates_shallow_marker_when_missing() {
    // Raw git2, not `Repo::init`: this is about opening a repo jig did
    // not create, which therefore has no marker yet.
    let tmp = TempDir::new().unwrap();
    git2::Repository::init(tmp.path()).unwrap();
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
    let repo = init_repo(tmp.path());

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

/// `find_valid_start_point` (private) falls back from `<base>` to
/// `origin/<base>` and gives up rather than inventing a start point. Both
/// halves are observable here.
#[test]
fn a_base_branch_that_resolves_nowhere_is_reported_missing() {
    let tmp = TempDir::new().unwrap();
    init_repo(tmp.path());
    let repo = Repo::open(tmp.path()).unwrap();

    // No remote, so "origin/main" resolves to nothing — and a base that
    // already names a remote gets no second guess.
    let result = repo.create_worktree(&Branch::new("feat/x"), &Branch::new("origin/main"));
    assert!(
        matches!(result, Err(GitError::BranchNotFound(ref s)) if s == "origin/main"),
        "expected BranchNotFound, got {result:?}"
    );
}

#[test]
fn a_local_base_branch_is_a_valid_start_point() {
    let tmp = TempDir::new().unwrap();
    init_repo(tmp.path());
    let repo = Repo::open(tmp.path()).unwrap();

    let path = repo
        .create_worktree(&Branch::new("feat/x"), &Branch::new("main"))
        .expect("local main is a valid start point");
    assert!(path.join(".git").exists());
}
