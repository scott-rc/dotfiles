---
name: git
description: Handles git commits, pushes, PRs, rebases, CI triage and monitoring, code review, branch splitting with stacked PRs via git-spice, and GitHub interactions -- use when the user asks to commit, push, amend, squash, rebase, create or update PRs, fix CI, review code, split a branch into stacked PRs, navigate stacks, or sync.
argument-hint: "[commit | squash | push | rebase | fix | correct | split | stack | sync] [context]"
---

# Git Operations

## Current State
- Branch: !`git branch --show-current`
- Status: !`git status --short`

Route to the appropriate operation based on user intent.

## Operations

### Commit
Commit outstanding changes with a well-formatted message. Also handles amend (fold changes into the last commit) -- see the Amend mode in `operations/commit.md`.
See operations/commit.md for detailed instructions.

### Squash
Squash all commits on the current branch into a single commit.
See operations/squash.md for detailed instructions.

### Rebase
Fetch latest and rebase onto base branch.
See operations/rebase.md for detailed instructions.

### Push
Push commits and create/update PR with title/description per guidelines. Also handles refresh description (update PR description without new commits) -- see the Refresh Description mode in `operations/push.md`.
See operations/push.md for detailed instructions.

### Fix
Auto-detect and fix CI failures, unresolved review threads, and PR description quality issues.
See operations/fix.md for detailed instructions.

### Correct
Propagate a user correction about what a change does to all affected artifacts (commit message, branch context, changesets, PR title/description).
See operations/correct.md for detailed instructions.

### Split
Split a large branch into stacked branches for easier code review. Analyzes the diff, proposes a stack grouped by concern, creates branches, and opens PRs.
See operations/split.md for detailed instructions.

### Stack
Navigate and manage stacked branches tracked by git-spice — move up/down, reorder, restack, list, track/untrack branches.
See operations/stack.md for detailed instructions.

### Sync
Fetch latest, clean up merged branches, and restack the stack.
See operations/sync.md for detailed instructions.

## Combined Operations

Multi-operation sequences and ambiguous phrasings that need explicit routing:

- **"commit this"** / **"commit these changes"** / **"commit my changes"** / **"commit what I changed"** → Commit (session-scoped: session files only, no ask)
- **"commit this and push"** / **"commit these changes and push"** → Commit (session-scoped), then Push
- **"commit and push"** → Commit (default scope), then Push
- **"amend and push"** → Commit (amend mode), then push
- **"squash and push"** → Squash, then push (push's uncommitted-changes check is redundant after squash)
- **"squash and update description"** / **"squash and update PR"** → Squash through Report (skip push offer), then Push (Refresh Description mode). Set `context` to note the squash. After refresh, offer force push since history was rewritten.
- **"push, then monitor"** → Push, then advise `/loop 2m /git fix`
- **"rerun, then monitor"** → Run `~/.claude/skills/git/scripts/rerun.sh`, then advise `/loop 2m /git fix`
- **"review and push"** / **"fix reviews and push"** → Fix, then push
- **"fix CI"** / **"debug CI"** / **"why is CI failing"** → Fix (not check-ci)
- **"address review comments"** / **"fix review feedback"** / **"fix bugbot comments"** → Fix
- **"update the before/after"** / **"edit the PR body"** / **"change part of the description"** → Push (Refresh Description mode) (all PR body modifications, even targeted section edits, go through pr-writer)
- **"that's not what this does"** / **"those were introduced in this PR"** / **"that flag doesn't exist"** / **"fix the commit message"** → Correct (propagates to all artifacts, not just the one being discussed)
- **"split"** / **"split this branch"** / **"split for review"** / **"stack this"** → Split
- **"split and push"** → Split (the split flow already includes pushing each branch and creating stacked PRs)
- **"sync"** / **"pull latest"** / **"update from main"** → Sync
- **"go up"** / **"next branch"** / **"go down"** / **"previous branch"** → Stack (navigate)
- **"reorder"** / **"move this branch"** / **"put this on top of X"** / **"rebase X after Y"** / **"rebase onto"** / **"change the base"** → Stack (reorder)
- **"restack"** / **"update the stack"** / **"restack upstack"** → Stack (restack)
- **"show stack"** / **"list branches"** / **"where am I"** → Stack (list)
- **"track this branch"** → Stack (track)

## Monitoring

Use `/loop 2m /git fix` to continuously monitor and fix CI failures and review threads. Each tick fires the Fix operation, which auto-detects what needs attention (CI failures, unresolved threads, description quality, or any combination) and handles it. The loop is session-scoped and auto-expires after 3 days.

## References

Reference files:
- references/git-patterns.md - Shared patterns: base branch detection, dotfiles exception, main branch protection, fetch safety, scope verification, script paths, local fix commands, git-spice integration
- references/git-spice-cli.md - git-spice (`gs`) CLI quick reference: commands, flags, and configuration options
- references/github-text.md - Universal formatting rules for all outbound text: commit messages, PR titles/descriptions, review comments (ASCII only, backtick code refs, safe posting)
- references/pr-writer-rules.md - Rules for callers that spawn the pr-writer agent
- references/bulk-threads.md - Threshold and pattern for handling bulk review threads via Explore subagent (used by Fix operation)
- references/commit-message-format.md - Commit message format rules (shared by inline commit path and committer agent)
- references/buildkite-handling.md - Buildkite log fetching, umbrella check handling, and auto-retry detection (used by Fix operation)

Scripts:
- scripts/get-pr-comments.sh - Fetches unresolved PR review threads; `--unreplied` filters to threads needing a reply, `--count` prints just the integer count, `--summary` prints a compact summary (one header line plus one line per thread) (used by Fix operation)
- scripts/get-failed-runs.sh - Retrieves run database IDs for failed CI checks on a branch (used by Fix operation)
- scripts/sanitize.sh - In-place ASCII text sanitizer with optional mode rules (`--commit-msg`, `--title`); used by committer, pr-writer, and inline commit paths
- scripts/check-ci.sh - Checks CI status for the current branch and prints a grouped summary (failed/pending/passed)
- scripts/rerun.sh - Re-triggers the most recent failed CI run on the current branch with fallback to full rerun
- scripts/branch-context-path.sh - Prints the branch context file path for the current branch (`./tmp/branches/<sanitized-branch>/context.md`)
