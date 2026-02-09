---
name: git
description: Handles all git operations and version control tasks including commits, pushes, rebases, merges, squashes, worktrees, branches, PRs, diffs, stashes, cherry-picks, resets, and CI checks.
compatibility:
  - gh CLI (GitHub CLI) for PR and CI operations
---

# Git Operations

Handle git operations based on user request. Determine which operation is needed:

## Operations

### Commit
Commit outstanding changes with a well-formatted message.
See [commit.md](commit.md) for detailed instructions.

### Squash
Squash all commits on the current branch into a single commit.
See [squash.md](squash.md) for detailed instructions.

### Rebase
Pull trunk and rebase the current branch.
See [rebase.md](rebase.md) for detailed instructions.

### Push
Push commits and create/update PR, syncing title/description with first commit.
See [push.md](push.md) for detailed instructions.

### CI
Check GitHub Actions status and fetch failure logs to debug CI issues.
See [ci.md](ci.md) for detailed instructions.

### Worktree
Create a new git worktree for a task or convert an existing branch.
See [worktree.md](worktree.md) for detailed instructions.

### Clean
Remove worktrees whose branches have been merged to main.
See [clean-worktrees.md](clean-worktrees.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"commit and push"** → Run commit operation, then push operation
- **"make a PR"** / **"open a PR"** → Same as push (push handles PR creation)
- **"sync"** / **"update branch"** → Same as rebase operation
- **"fix PR description"** / **"update PR"** / **"sync PR"** → Run push.md step 7 only (sync PR title/description with first commit)

**Important**: For each operation, read and follow its detailed instruction file (e.g., commit.md, push.md). These files contain required steps that must not be skipped.
