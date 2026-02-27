# Squash

Squash all commits on the current branch into a single commit.

## Instructions

1. **Fetch latest from remote**: `git fetch origin`

2. **Detect base branch**: Run `fish -c 'gbb'` to get the base branch name.

3. **List commits to squash**: `git log origin/<base>..HEAD --oneline`

4. **Commit uncommitted changes**: Check for uncommitted changes. If changes exist, spawn the `committer` agent with no additional prompt. If clean, skip to step 5.

5. **Rebase onto base branch** so all comparisons against `origin/<base>` are accurate: `git rebase origin/<base>`. If conflicts occur, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git rebase --abort`)". Only run `git rebase --abort` if the user picks the abort option.

6. **Analyze all commits** to understand what work was done and why: `git log origin/<base>..HEAD --format="%h %s%n%b"`. Note the overall purpose for crafting the squash message.

7. **Verify scope before squashing**: MUST follow the scope verification pattern in references/git-patterns.md.

8. **Confirm before squashing**:
   - Show the commits that will be squashed (from step 3)
   - Show what files will be in the final commit: `git diff --stat origin/<base> HEAD`
   - MUST ask the user to confirm before proceeding

9. **Squash into a single commit**: `git reset --soft origin/<base>`. Then spawn the `committer` agent with prompt: "Squash commit. Prior commits: <commit list from step 6>."

10. **Report**: Show the squashed commit hash and message. If the branch tracks a remote, present options via AskUserQuestion: "Push" or "Skip". If the user accepts, run the Push operation. If no remote tracking branch, just report the result.
