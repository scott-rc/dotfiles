# Squash

Squash all commits on the current branch into a single commit.

## Instructions

1. **Fetch latest from remote**: `git fetch origin`

2. **Detect base branch**: Detect base branch per references/git-patterns.md.

3. **List commits to squash**: `git log origin/<base>..HEAD --oneline`

4. **Commit uncommitted changes**: Check for uncommitted changes. If changes exist, spawn the `committer` agent with no additional prompt. If clean, skip to step 5.

5. **Rebase onto base branch** so all comparisons against `origin/<base>` are accurate: `git rebase origin/<base>`. If conflicts occur, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git rebase --abort`)". Only run `git rebase --abort` if the user picks the abort option.

6. **Verify scope before squashing**: MUST follow the scope verification pattern in references/git-patterns.md.

7. **Confirm before squashing**:
   - Re-list commits after the rebase: `git log origin/<base>..HEAD --oneline` (the rebase in step 5 may have changed the commit count)
   - Show what files will be in the final commit: `git diff --stat origin/<base> HEAD`
   - MUST ask the user to confirm before proceeding

8. **Squash into a single commit**: `git reset --soft origin/<base>`. Then spawn the `committer` agent with prompt: "Squash commit."

9. **Report**: Show the squashed commit hash and message. If the branch tracks a remote, present options via AskUserQuestion: "Push" or "Skip". If the user accepts, run the Push operation. If no remote tracking branch, just report the result.
