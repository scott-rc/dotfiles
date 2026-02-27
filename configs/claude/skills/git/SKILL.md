---
name: git
description: Handles git commits, pushes, PRs, rebases, worktrees, CI triage and monitoring, code review, and GitHub interactions.
argument-hint: "[operation or intent]"
---

# Git Operations

## Current State
- Branch: !`git branch --show-current`
- Status: !`git status --short`
- Recent commits: !`git log --oneline -5`

Route to the appropriate operation based on user intent.

## Operations

### Commit
Commit outstanding changes with a well-formatted message.
See operations/commit.md for detailed instructions.

### Amend
Fold outstanding changes into the last commit.
See operations/amend.md for detailed instructions.

### Squash
Squash all commits on the current branch into a single commit.
See operations/squash.md for detailed instructions.

### Rebase
Fetch latest and rebase onto base branch.
See operations/rebase.md for detailed instructions.

### Push
Push commits and create/update PR with title/description per guidelines.
See operations/push.md for detailed instructions.

### Worktree
Create a new git worktree for a task or convert an existing branch.
See operations/worktree.md for detailed instructions.

### Clean Worktrees
Remove merged, squash-merged, and orphaned worktrees via the `gwc` fish function.
See operations/clean-worktrees.md for detailed instructions.

### Fix CI
Check CI status, fetch failure logs, identify root cause, and fix the issues.
See operations/fix-ci.md for detailed instructions.

### Rerun
Re-trigger failed CI jobs.

1. **Find failed run**: `gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName`. If none, inform user and stop.
2. **Rerun**: `gh run rerun <run-id> --failed` (fall back to `gh run rerun <run-id>` if unsupported)
3. **Confirm**: `gh run view <run-id> --json status`, report run ID and status. If the returned status is still `failed` (not `queued` or `in_progress`), report the error to the user.
4. Offer to run Watch to monitor results

### Watch
Monitor CI and review threads on the current PR, automatically triaging failures, fixing issues, and pushing updates.
See operations/watch.md for detailed instructions.

### Review
Fetch unresolved PR review threads and fix the issues reviewers described.
See operations/review.md for detailed instructions.

### Update Description
Rewrite the PR title and description to match current changes per guidelines.
See operations/update-description.md for detailed instructions.

### Submit Review
Submit a PR review (approve, request changes, or comment) with optional inline comments.
See operations/submit-review.md for detailed instructions.

### Reply
Fetch unreplied PR review threads and draft responses for user approval, or post a specific comment.
See operations/reply.md for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"commit and push"** → Run commit operation, then push operation
- **"amend"** / **"fold into last commit"** / **"add to last commit"** → Run amend operation
- **"amend and push"** → Run amend operation, then push operation
- **"squash and push"** → Run squash operation, then push operation
- **"make a PR"** / **"open a PR"** → Same as push (push handles PR creation)
- **"sync"** / **"update branch"** → Same as rebase operation
- **"check CI"** / **"CI status"** → Run fix-ci operation in status-only mode: fetch CI state and report pass/fail/pending counts, but do not triage failures or apply any fixes
- **"fix CI"** / **"debug CI"** / **"why is CI failing"** → Run fix-ci operation (full, including fix)
- **"rerun CI"** / **"retry CI"** / **"re-trigger"** → Run rerun operation
- **"rerun and watch"** → Run rerun operation, then watch operation to monitor new status
- **"watch CI"** / **"monitor PR"** / **"sleep and watch"** / **"watch"** → Run watch operation
- **"push and watch"** → Run push operation, then watch operation
- **"address review comments"** / **"fix review feedback"** / **"fix bugbot comments"** → Run review operation
- **"fix PR description"** / **"update PR"** / **"sync PR"** → Run update-description operation
- **"reply to this comment"** / **"post a comment"** / **"answer this question on the PR"** → Run reply operation
- **"reply to reviews"** / **"respond to feedback"** → Run reply operation (auto-discover mode)
- **"approve"** / **"approve this PR"** / **"LGTM"** / **"submit review"** → Run submit-review operation
- **"approve and comment"** / **"approve with comments"** / **"approve and add comments"** → Run submit-review operation
- **"request changes"** / **"block this PR"** / **"needs work"** → Run submit-review operation
- **"review and push"** / **"fix reviews and push"** → Run review operation, then push operation

## References

Reference files:
- references/git-patterns.md - Shared patterns: base branch detection, dotfiles exception, main branch protection, fetch safety, scope verification, script paths, local fix commands
- references/github-text.md - Universal formatting rules for all GitHub-facing text (ASCII only, backtick code refs, safe posting)
- references/pr-writer-rules.md - Rules for callers that spawn the pr-writer agent
- references/bulk-threads.md - Threshold and pattern for handling bulk review threads via Explore subagent (used by Review and Reply operations)
- references/buildkite-handling.md - Buildkite log fetching, umbrella check handling, and auto-retry detection (used by Watch operation)
- references/watch-subops.md - State file format and monitoring loop protocol for the watch loop

Scripts:
- scripts/get-pr-comments.sh - Fetches unresolved PR review threads; `--unreplied` flag filters to threads needing a reply (used by Review, Reply, and Watch operations)
- scripts/poll-pr-status.sh - Combined CI + review thread poll for the watch loop; returns compact JSON with exit condition (used by Watch operation)
- scripts/get-failed-runs.sh - Retrieves run database IDs for failed CI checks on a branch (used by Watch operation)
