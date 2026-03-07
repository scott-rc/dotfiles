# Squash

Squash all commits on the current branch into a single commit.

## Instructions

1. **Fetch latest from remote**: `git fetch origin`

2. **Detect base branch**: Detect base branch per references/git-patterns.md.

3. **List commits to squash**: `git log origin/<base>..HEAD --oneline`. Also capture full commit messages for step 8: `git log origin/<base>..HEAD --format=%B`.

4. **Commit uncommitted changes**: Check for uncommitted changes. If clean, skip to step 5. If changes exist, evaluate complexity (same criteria: session file set known, 5 or fewer files, 100 or fewer lines changed). If simple, commit per the Inline Commit Procedure in references/commit-message-format.md. If complex, spawn the `committer` agent with no additional prompt.

5. **Rebase onto base branch** so all comparisons against `origin/<base>` are accurate: `git rebase origin/<base>`. If conflicts occur, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git rebase --abort`)". Only run `git rebase --abort` if the user picks the abort option.

6. **Verify scope before squashing**: MUST follow the scope verification pattern in references/git-patterns.md.

7. **Confirm before squashing**:
   - Re-list commits after the rebase: `git log origin/<base>..HEAD --oneline` (the rebase in step 5 may have changed the commit count)
   - Show what files will be in the final commit: `git diff --stat origin/<base>...HEAD`
   - MUST ask the user to confirm before proceeding

8. **Squash into a single commit**: `git reset --soft origin/<base>`. Read the branch context file if it exists (path per references/git-patterns.md "Branch Context File"). Spawn the `committer` agent with prompt: "Squash commit. Original commit messages:\n<full commit messages captured in step 3>". If branch context exists, append: "\nBranch purpose:\n<branch context contents>"

9. **Report**: Show the squashed commit hash and message. If the branch tracks a remote, present options via AskUserQuestion: "Push" or "Skip". If the user accepts, run the Push operation. Note: Push's uncommitted-changes check is redundant after squash -- skip it. If no remote tracking branch, just report the result.
