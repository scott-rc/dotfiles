---
name: git
description: Git operations including commit, squash, rebase, worktrees, and PR management.
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
Pull trunk and rebase the current branch (and stack) using git-spice.
See [rebase.md](rebase.md) for detailed instructions.

### Push
Push commits and create/update PR, syncing title/description with first commit.
See [push.md](push.md) for detailed instructions.

### Status
Show current branch status, changes, and stack info.
See [status.md](status.md) for detailed instructions.

### CI
Check GitHub Actions status and fetch failure logs to debug CI issues.
See [ci.md](ci.md) for detailed instructions.

### Worktree
Create a new git worktree for a task or convert an existing branch.
See [worktree.md](worktree.md) for detailed instructions.

### Clean
Remove worktrees whose branches have been merged to main.
See [clean-worktrees.md](clean-worktrees.md) for detailed instructions.
