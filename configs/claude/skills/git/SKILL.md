---
name: git
description: Handles git commits, amends, pushes, rebases, squashes, worktrees, CI fixes and reruns, PR review submissions, PR review comments, PR description updates, and GitHub comment/reply posting. Use when the user asks to commit, amend, push, create or update a PR, rebase, squash commits, manage worktrees, check CI, fix CI, rerun CI, approve a PR, request changes, submit a review, address review comments, fix review feedback, fix bugbot comments, update PR description, sync PR, reply to a PR comment, reply to reviews, or post a comment on GitHub.
---

# Git Operations

Route to the appropriate operation based on user intent.

## GitHub Text Rule

**Any text sent to GitHub** (PR descriptions, PR comments, review replies, issue comments, etc.) MUST follow the "All GitHub Text" section of [pr-guidelines.md](pr-guidelines.md) -- ASCII only, no em dashes, no curly quotes. This applies to ALL operations below and to ad-hoc GitHub interactions.

## Operations

### Commit
Commit outstanding changes with a well-formatted message.
See [commit.md](commit.md) for detailed instructions.

### Amend
Fold outstanding changes into the last commit.
See [amend.md](amend.md) for detailed instructions.

### Squash
Squash all commits on the current branch into a single commit.
See [squash.md](squash.md) for detailed instructions.

### Rebase
Fetch latest and rebase onto base branch.
See [rebase.md](rebase.md) for detailed instructions.

### Push
Push commits and create/update PR with title/description per guidelines.
See [push.md](push.md) for detailed instructions.

### Worktree
Create a new git worktree for a task or convert an existing branch.
See [worktree.md](worktree.md) for detailed instructions.

### Clean
Remove merged, squash-merged, and orphaned worktrees via the `gwc` fish function.
See [clean-worktrees.md](clean-worktrees.md) for detailed instructions.

### Fix CI
Check CI status, fetch failure logs, identify root cause, and fix the issues.
See [fix-ci.md](fix-ci.md) for detailed instructions.

### Rerun
Re-trigger failed CI jobs.
See [rerun.md](rerun.md) for detailed instructions.

### Review
Fetch unresolved PR review threads and fix the issues reviewers described.
See [review.md](review.md) for detailed instructions.

### Update Description
Rewrite the PR title and description to match current changes per guidelines.
See [update-description.md](update-description.md) for detailed instructions.

### Submit Review
Submit a PR review (approve, request changes, or comment) with optional inline comments.
See [submit-review.md](submit-review.md) for detailed instructions.

### Reply
Fetch unreplied PR review threads and draft responses for user approval, or post a specific comment.
See [reply.md](reply.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"commit and push"** → Run commit operation, then push operation
- **"amend"** / **"fold into last commit"** / **"add to last commit"** → Run amend operation
- **"amend and push"** → Run amend operation, then push operation
- **"squash and push"** → Run squash operation, then push operation
- **"make a PR"** / **"open a PR"** → Same as push (push handles PR creation)
- **"sync"** / **"update branch"** → Same as rebase operation
- **"check CI"** / **"CI status"** / **"why is CI failing"** / **"debug CI"** / **"fix CI"** → Run fix-ci operation
- **"rerun CI"** / **"retry CI"** / **"re-trigger"** → Run rerun operation
- **"rerun and watch"** → Run rerun operation, then fix-ci operation to monitor new status
- **"address review comments"** / **"fix review feedback"** / **"fix bugbot comments"** → Run review operation
- **"fix PR description"** / **"update PR"** / **"sync PR"** → Run update-description operation
- **"reply to this comment"** / **"post a comment"** / **"answer this question on the PR"** → Run reply operation
- **"reply to reviews"** / **"respond to feedback"** → Run reply operation (auto-discover mode)
- **"approve"** / **"approve this PR"** / **"LGTM"** / **"submit review"** → Run submit-review operation
- **"approve and comment"** / **"approve with comments"** / **"approve and add comments"** → Run submit-review operation
- **"request changes"** / **"block this PR"** / **"needs work"** → Run submit-review operation
- **"review and push"** / **"fix reviews and push"** → Run review operation, then push operation

## References

These files are referenced by the operation instructions above:

- [git-patterns.md](git-patterns.md) - Shared patterns: base branch detection, dotfiles exception, main branch protection, fetch safety, scope verification
- [commit-guidelines.md](commit-guidelines.md) - Commit message format, structure, and examples
- [pr-guidelines.md](pr-guidelines.md) - Formatting rules for all GitHub-facing text (PR descriptions, comments, reviews)
- `scripts/get-pr-comments.sh` - Fetches unresolved PR review threads (used by Review operation)
- `scripts/get-unreplied-comments.sh` - Fetches unreplied PR review threads (used by Reply operation)

Scripts require the skill to be installed at `~/.claude/skills/git/` (set up by `apply.sh`).
