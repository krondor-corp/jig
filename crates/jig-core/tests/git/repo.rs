//! `Repo` against real repositories on disk.

use std::path::{Path, PathBuf};

use crate::common::seeded_repo as init_repo;
use jig_core::exec::Timeout;
use jig_core::git::{Branch, GitError, Repo, Worktree};
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
fn opening_a_repo_leaves_no_shallow_marker_behind() {
    // jig used to create an empty `.git/shallow` in every repo it touched,
    // to quiet a libgit2 stat error that turned out to be a stale message
    // from an unrelated failure. Git reads the file's *existence* as "this
    // is a shallow clone", so writing one made every repo lie about itself.
    let tmp = TempDir::new().unwrap();
    git2::Repository::init(tmp.path()).unwrap();

    let _repo = Repo::open(tmp.path()).unwrap();

    assert!(
        !tmp.path().join(".git").join("shallow").exists(),
        "opening a repo must not create a shallow marker"
    );
}

#[test]
fn opening_a_repo_preserves_a_real_shallow_file() {
    let tmp = TempDir::new().unwrap();
    let repo = init_repo(tmp.path());
    let head_oid = repo.head().unwrap().target().unwrap().to_string();
    let shallow_contents = format!("{}\n", head_oid);
    drop(repo);

    let shallow = tmp.path().join(".git").join("shallow");
    std::fs::write(&shallow, shallow_contents.as_bytes()).unwrap();

    let _repo = Repo::open(tmp.path()).unwrap();

    assert_eq!(
        std::fs::read(&shallow).unwrap(),
        shallow_contents.as_bytes(),
        "a genuinely shallow clone's graft points must not be touched"
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

#[test]
fn checked_out_branch_follows_a_rename_while_the_worker_name_does_not() {
    let tmp = TempDir::new().unwrap();
    let _ = init_repo(tmp.path());
    let repo = Repo::open(tmp.path()).unwrap();
    let path = repo
        .create_worktree(&"al/triage-fixes".into(), &"main".into())
        .unwrap();

    git2::Repository::open(&path)
        .unwrap()
        .find_branch("al/triage-fixes", git2::BranchType::Local)
        .unwrap()
        .rename("fix/triage-timeout-and-budget", false)
        .unwrap();

    let wt = Worktree::open(&path).unwrap();

    // The worker keeps its identity: event log, mux window, prune target.
    assert_eq!(&*wt.branch_name(), "al/triage-fixes");
    // GitHub must be asked about the branch, which has moved on.
    assert_eq!(
        wt.checked_out_branch().as_deref(),
        Some("fix/triage-timeout-and-budget"),
        "PR lookups must follow the branch, not the folder"
    );
}

#[test]
fn a_detached_worktree_has_no_checked_out_branch() {
    let tmp = TempDir::new().unwrap();
    let _ = init_repo(tmp.path());
    let repo = Repo::open(tmp.path()).unwrap();
    let path = repo
        .create_worktree(&"al/detached".into(), &"main".into())
        .unwrap();

    let wt_git = git2::Repository::open(&path).unwrap();
    let head = wt_git.head().unwrap().target().unwrap();
    wt_git.set_head_detached(head).unwrap();

    assert_eq!(
        Worktree::open(&path).unwrap().checked_out_branch(),
        None,
        "a detached HEAD names no branch to look up"
    );
}

#[test]
fn remove_works_after_the_branch_is_renamed_inside_the_worktree() {
    let tmp = TempDir::new().unwrap();
    let _ = init_repo(tmp.path());
    let repo = Repo::open(tmp.path()).unwrap();
    let path = repo
        .create_worktree(&"feat/upload-neverthrow".into(), &"main".into())
        .unwrap();

    // Renaming a branch inside a worktree is ordinary; the worktree's
    // registration keeps the name it was created with.
    let wt_git = git2::Repository::open(&path).unwrap();
    wt_git
        .find_branch("feat/upload-neverthrow", git2::BranchType::Local)
        .unwrap()
        .rename("feat/membership-error", false)
        .unwrap();

    Worktree::open(&path).unwrap().remove(true).unwrap();

    assert!(!path.exists(), "working directory should be gone");
    assert!(
        repo.list_worktrees().unwrap().is_empty(),
        "registration should be gone"
    );
}

/// `git push` must run in the worktree whose branch is being pushed.
///
/// Git runs hooks against the working tree it is invoked in, and
/// `clone_path` is the main clone even for a linked worktree. Pushing from
/// there ran a repo's `pre-push` hook over whatever the clone had checked
/// out, so a hook that type-checks the tree checked `dev` instead of the
/// branch being pushed — failing `jig pr` while a plain `git push` from the
/// worktree passed.
///
/// The hook writes its `pwd`, which is the only way to see where git ran.
#[test]
fn push_runs_hooks_in_the_worktree_being_pushed() {
    let tmp = TempDir::new().unwrap();
    let origin_dir = tmp.path().join("origin");
    std::fs::create_dir_all(&origin_dir).unwrap();
    git2::Repository::init_bare(&origin_dir).unwrap();

    let clone_dir = tmp.path().join("clone");
    std::fs::create_dir_all(&clone_dir).unwrap();
    let git = init_repo(&clone_dir);
    git.remote("origin", origin_dir.to_str().unwrap()).unwrap();

    // A worktree on its own branch, beside the clone.
    let head = git.head().unwrap().peel_to_commit().unwrap();
    let branch = git.branch("feature/x", &head, false).unwrap();
    let wt_path = tmp.path().join("wt");
    let mut opts = git2::WorktreeAddOptions::new();
    opts.reference(Some(branch.get()));
    git.worktree("wt", &wt_path, Some(&opts)).unwrap();

    // A pre-push hook that records the directory git ran it in.
    let witness = tmp.path().join("where.txt");
    let hooks = clone_dir.join(".git/hooks");
    std::fs::create_dir_all(&hooks).unwrap();
    let hook = hooks.join("pre-push");
    std::fs::write(&hook, format!("#!/bin/sh\npwd > {}\n", witness.display())).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let repo = Repo::open(&wt_path).unwrap();
    repo.push_branch(&Branch::from("feature/x"), Timeout::QUICK)
        .unwrap();

    let ran_in = std::fs::read_to_string(&witness).expect("pre-push hook did not run");
    let ran_in = std::fs::canonicalize(ran_in.trim()).unwrap();
    assert_eq!(
        ran_in,
        std::fs::canonicalize(&wt_path).unwrap(),
        "the hook ran in the clone, so it checked the wrong branch"
    );
}
