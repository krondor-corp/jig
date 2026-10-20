---
description: Push current branch and create a draft PR. Use when ready to share work for review or collaborate on a branch.
allowed-tools:
  - Bash(jig:*)
  - Read
  - Glob
  - Grep
---

Create a draft pull request for the current branch.

## Steps

1. Get the current branch name:
   ```
   git branch --show-current
   ```

2. Check for uncommitted changes:
   ```
   git status --porcelain
   ```
   If there are uncommitted changes (modified, added, or untracked files):
   a. Run the project's formatter/linter if applicable (check CLAUDE.md or docs/index.md for commands)
   b. Stage all changes: `git add -A`
   c. Create a commit with a descriptive message based on the changes
   d. Use conventional commit format (feat:, fix:, docs:, refactor:, test:, chore:)

3. Check if the branch has an upstream:
   ```
   git status -sb
   ```
   - If no upstream: `git push -u origin <branch>`
   - If upstream exists: `git push`

4. Gather context — commits unique to this branch:
   ```
   git log main..HEAD --oneline
   ```
   (Use `master` if the repo has no `main`.)

5. Create a draft PR:
   ```
   jig pr create --title "<title>" --body "<body>"
   ```
   - Title: descriptive of what the branch accomplishes
   - Body: summarize ALL changes based on the commits
   - `jig pr` resolves the base branch itself, including the parent of a
     stacked worktree. Do not pass a base, and do not use `gh pr create` —
     it skips that resolution and targets the wrong base.

6. Return the PR URL to the user.

## Important

- **Commit ALL uncommitted changes** before pushing — don't leave anything behind
- If the linter/formatter finds issues, fix them before committing
