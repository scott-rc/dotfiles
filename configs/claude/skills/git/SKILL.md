---
name: git
description: Handles git commits, PRs, rebases, worktrees, CI fixes, CI monitoring, review submissions, and GitHub interactions — use when the user asks to commit, push, rebase, fix CI, watch CI, review, or manage worktrees.
argument-hint: "[operation or intent]"
---

# Git Operations

## Current State
- Branch: !`git branch --show-current`
- Status: !`git status --short`
- Recent commits: !`git log --oneline -5`

Route to the appropriate operation based on user intent.

## GitHub Text Rule

**Any text sent to GitHub** (PR descriptions, PR comments, review replies, issue comments, etc.) MUST follow the "All GitHub Text" section of [pr-guidelines.md](pr-guidelines.md) -- ASCII only, no em dashes, no curly quotes. This applies to ALL operations below and to ad-hoc GitHub interactions.

## Operations

### Commit
Commit outstanding changes with a well-formatted message.

1. **Check branch protection** per [git-patterns.md](git-patterns.md). If on main/master and not dotfiles, present branch options via AskUserQuestion. If chosen, create and switch to the branch before committing.
2. **Delegate to the `committer` agent**. Pass no additional prompt -- the agent gathers context, drafts a message, stages, and commits autonomously.
3. **If the agent returns `needs-user-input`** (mixed concerns): present the groups from `## Cohesion` as AskUserQuestion options. Re-invoke the agent with: "Stage and commit only these files: `<file list>`".
4. **Report**: show the commit hash and title from the agent's `## Commit` section.

### Amend
Fold outstanding changes into the last commit.
See [amend.md](amend.md) for detailed instructions.

### Squash
Squash all commits on the current branch into a single commit.
See [squash.md](squash.md) for detailed instructions.

### Rebase
Fetch latest and rebase onto base branch.

1. **Fetch**: `git fetch origin`
2. **Detect base branch**: `fish -c 'gbb'`
3. **Rebase**: `git rebase origin/<base>`
4. **If conflicts**: list conflicting files (`git diff --name-only --diff-filter=U`), report to user, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase"
5. **If success**: verify scope per [git-patterns.md](git-patterns.md), show commit count: `git rev-list --count origin/<base>..HEAD`

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

1. **Find failed run**: `gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName`. If none, inform user and stop.
2. **Rerun**: `gh run rerun <run-id> --failed` (fall back to `gh run rerun <run-id>` if unsupported)
3. **Confirm**: `gh run view <run-id> --json status`, report run ID and status
4. Offer to run Watch to monitor results

### Watch
Monitor CI and review threads on the current PR, automatically triaging failures, fixing issues, and pushing updates.
See [watch.md](watch.md) for detailed instructions.

### Review
Fetch unresolved PR review threads and fix the issues reviewers described.
See [review.md](review.md) for detailed instructions.

### Update Description
Rewrite the PR title and description to match current changes per guidelines.

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.
2. **Delegate to `pr-writer` agent** with: mode `update`, base_branch (detect per [git-patterns.md](git-patterns.md)), pr_number.
3. **Report**: confirm update, show PR URL.

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

These files are referenced by the operation instructions above:

- [git-patterns.md](git-patterns.md) - Shared patterns: base branch detection, dotfiles exception, main branch protection, fetch safety, scope verification
- [pr-guidelines.md](pr-guidelines.md) - Formatting rules for all GitHub-facing text (PR descriptions, comments, reviews)
- [watch-subops.md](watch-subops.md) - Procedures for handling review threads and CI failures during the watch loop
- `scripts/get-pr-comments.sh` - Fetches unresolved PR review threads; `--unreplied` flag filters to threads needing a reply (used by Review, Reply, and Watch operations)
- `scripts/poll-pr-status.sh` - Combined CI + review thread poll for the watch loop; returns compact JSON with exit condition (used by Watch operation)
- `scripts/get-failed-runs.sh` - Retrieves run database IDs for failed CI checks on a branch (used by Watch operation via watch-subops.md)

Scripts require the skill to be installed at `~/.claude/skills/git/`. All script paths are listed in [git-patterns.md](git-patterns.md).
