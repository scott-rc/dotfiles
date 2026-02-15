---
name: git
description: Handles git commits, pushes, rebases, squashes, worktrees, PRs, CI checks, and PR reviews with opinionated workflows. Use when the user asks to commit, push, create or update a PR, rebase, squash commits, check CI status, address review comments, or manage git worktrees.
---

# Git Operations

Route to the appropriate operation based on user intent.

## Operations

### Commit
Commit outstanding changes with a well-formatted message.
See [commit.md](commit.md) for detailed instructions.

### Squash
Squash all commits on the current branch into a single commit.
See [squash.md](squash.md) for detailed instructions.

### Rebase
Fetch latest and rebase onto base branch.
See [rebase.md](rebase.md) for detailed instructions.

### Push
Push commits and create/update PR with title/description per guidelines.
See [push.md](push.md) for detailed instructions.

### CI
Check GitHub Actions status and fetch failure logs to debug CI issues.
See [ci.md](ci.md) for detailed instructions.

### Review
Fetch unresolved PR review threads and fix the issues reviewers described.
See [review.md](review.md) for detailed instructions.

### Worktree
Create a new git worktree for a task or convert an existing branch.
See [worktree.md](worktree.md) for detailed instructions.

### Clean
Remove worktrees whose branches have been merged to main.
See [clean-worktrees.md](clean-worktrees.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"commit and push"** → Run commit operation, then push operation
- **"squash and push"** → Run squash operation, then push operation
- **"make a PR"** / **"open a PR"** → Same as push (push handles PR creation)
- **"sync"** / **"update branch"** → Same as rebase operation
- **"fix PR description"** / **"update PR"** / **"sync PR"** → Run push.md step 7 only (rewrite PR title/description per guidelines)
- **"address review comments"** / **"fix review feedback"** / **"fix bugbot comments"** → Run review operation
- **"review and push"** / **"fix reviews and push"** → Run review operation, then push operation

**Important**: For each operation, read and follow its detailed instruction file (e.g., commit.md, push.md). These files contain required steps that must not be skipped.

## Dependencies

Requires `git`, `gh` (GitHub CLI), and `jq`.

## References

These files are referenced by the operation instructions above:

- [git-patterns.md](git-patterns.md) - Shared patterns: base branch detection, dotfiles exception, main branch protection, fetch safety, scope verification
- [commit-guidelines.md](commit-guidelines.md) - Commit message format, structure, and examples
- [pr-guidelines.md](pr-guidelines.md) - PR title and description format
