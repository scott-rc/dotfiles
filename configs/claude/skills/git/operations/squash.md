# Squash

Squash all commits on the current branch into a single commit.

## Instructions

1. **Fetch latest from remote**: `git fetch origin`

2. **Detect base branch**: Detect base branch per references/git-patterns.md.

3. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-patterns.md.

4. **List commits to squash**: `git log origin/<base>..HEAD --oneline`. Also capture full commit messages for step 9: `git log origin/<base>..HEAD --format=%B`.

5. **Commit uncommitted changes**: Check for uncommitted changes. If clean, skip to step 6. If changes exist, evaluate complexity (same criteria: session file set known, 5 or fewer files, 100 or fewer lines changed). If simple, commit per the Inline Commit Procedure in references/commit-message-format.md. If complex, spawn the `committer` agent with no additional prompt.

6. **Rebase onto base branch** so all comparisons against `origin/<base>` are accurate: `git-spice upstack restack`. If conflicts occur, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git-spice rebase abort`)". After resolving conflicts, use `git-spice rebase continue --no-prompt` to resume (auto-restacks). Only run `git-spice rebase abort` if the user picks the abort option.

7. **Verify scope before squashing**: MUST follow the scope verification pattern in references/git-patterns.md.

8. **Confirm before squashing**:
   - Re-list commits after the rebase: `git log origin/<base>..HEAD --oneline` (the rebase in step 6 may have changed the commit count)
   - Show what files will be in the final commit: `git diff --stat origin/<base>...HEAD`
   - MUST ask the user to confirm before proceeding

9. **Squash into a single commit**: Read the branch context file if it exists (path per references/git-patterns.md "Branch Context File"). Run `git-spice branch squash --no-prompt` to squash all branch commits into one (this also auto-restacks any upstack branches). Then spawn the `committer` agent with prompt: "Amend the squash commit message. Original commit messages:\n<full commit messages captured in step 4>". If branch context exists, append: "\nBranch purpose:\n<branch context contents>". The committer uses `git-spice commit amend --no-prompt` to apply the quality message.

10. **Report**: Show the squashed commit hash and message. If the branch tracks a remote, present options via AskUserQuestion: "Push" or "Skip". If the user accepts, run the Push operation. Note: Push's uncommitted-changes check is redundant after squash -- skip it. If no remote tracking branch, just report the result.
