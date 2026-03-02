---
name: git
description: Handles git commits, pushes, PRs, rebases, worktrees, CI triage and monitoring, code review, and GitHub interactions -- use when the user asks to commit, push, amend, squash, rebase, create or update PRs, fix CI, review code, or manage worktrees.
argument-hint: "[operation or intent]"
---

# Git Operations

## Current State
- Branch: !`git branch --show-current`
- Status: !`git status --short`

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

### Check CI
Check CI status and report results.
See operations/check-ci.md for detailed instructions.

### Fix CI
Fetch CI failure logs, triage via ci-triager, and fix issues via fix subagent.
See operations/fix-ci.md for detailed instructions.

### Rerun
Re-trigger failed CI jobs.
See operations/rerun.md for detailed instructions.

### Watch
Monitor CI and review threads on the current PR, automatically triaging failures, fixing issues, and pushing updates.
See operations/watch.md for detailed instructions.

### Fix Review
Fetch unresolved PR review threads and fix the issues reviewers described.
See operations/fix-review.md for detailed instructions.

### Update Description
Rewrite the PR title and description to match current changes per guidelines.
See operations/update-description.md for detailed instructions.

### Set Branch Context
Read or create the branch context file that captures the "why" for the current branch.
See operations/set-branch-context.md for detailed instructions.

### Submit Review
Submit a PR review (approve, request changes, or comment) with optional inline comments.
See operations/submit-review.md for detailed instructions.

### Reply
Fetch unreplied PR review threads and draft responses for user approval, or post a specific comment.
See operations/reply.md for detailed instructions.

### Correct
Propagate a user correction about what a change does to all affected artifacts (commit message, branch context, changesets, PR title/description).
See operations/correct.md for detailed instructions.

## Combined Operations

Multi-operation sequences and ambiguous phrasings that need explicit routing:

- **"commit and push"** → Commit, then push
- **"amend and push"** → Amend, then push
- **"squash and push"** → Squash, then push (push's uncommitted-changes check is redundant after squash)
- **"squash and update description"** / **"squash and update PR"** → Squash through Report (skip push offer), then update-description. Set `context` to note the squash. After update-description, offer force push since history was rewritten.
- **"push and watch"** → Push, then watch
- **"rerun and watch"** → Rerun, then watch
- **"review and push"** / **"fix reviews and push"** → Fix-review, then push
- **"fix CI"** / **"debug CI"** / **"why is CI failing"** → Fix-ci (not check-ci)
- **"address review comments"** / **"fix review feedback"** / **"fix bugbot comments"** → Fix-review (not reply)
- **"update the before/after"** / **"edit the PR body"** / **"change part of the description"** → Update Description (not github-writer -- all PR body modifications, even targeted section edits, go through pr-writer)
- **"that's not what this does"** / **"those were introduced in this PR"** / **"that flag doesn't exist"** / **"fix the commit message"** → Correct (propagates to all artifacts, not just the one being discussed)

## References

Reference files:
- references/git-patterns.md - Shared patterns: base branch detection, dotfiles exception, main branch protection, fetch safety, scope verification, script paths, local fix commands
- references/github-text.md - Universal formatting rules for all GitHub-facing text (ASCII only, backtick code refs, safe posting)
- references/pr-writer-rules.md - Rules for callers that spawn the pr-writer agent
- references/bulk-threads.md - Threshold and pattern for handling bulk review threads via Explore subagent (used by Fix Review and Reply operations)
- references/buildkite-handling.md - Buildkite log fetching, umbrella check handling, and auto-retry detection (used by Watch and Fix CI operations)
- references/watch-subops.md - State file format, monitoring loop protocol, and thread/CI failure handling for the watch loop

Scripts:
- scripts/get-pr-comments.sh - Fetches unresolved PR review threads; `--unreplied` flag filters to threads needing a reply (used by Fix Review and Reply operations)
- scripts/poll-pr-status.sh - Combined CI + review thread poll for the watch loop; returns compact JSON with exit condition (used by Watch operation)
- scripts/get-failed-runs.sh - Retrieves run database IDs for failed CI checks on a branch (used by Watch operation)
