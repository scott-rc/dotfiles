# Rebase

Fetch latest and rebase onto base branch.

## Instructions

1. **Fetch latest from remote**: Run `git fetch origin`.

2. **Detect base branch**: Detect base branch per references/git-patterns.md.

3. **Rebase onto base branch**:
   - **Git-spice check**: If the branch is tracked by git-spice (per references/git-patterns.md Git-Spice > Detection and Git-Spice > Tracked Branch Check), use `gs upstack restack` instead. This rebases the current branch AND all branches above it onto their updated bases. Continue to step 4 for conflict handling (git-spice surfaces conflicts the same way).
   - Otherwise: run `git rebase origin/<base>`.

4. **If conflicts**: List conflicting files (`git diff --name-only --diff-filter=U`), report to user, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git rebase --abort`)". Only run `git rebase --abort` if the user picks the abort option.

5. **If success**: Verify scope per references/git-patterns.md.

6. **Report**: Show commit count: `git rev-list --count origin/<base>..HEAD`.

