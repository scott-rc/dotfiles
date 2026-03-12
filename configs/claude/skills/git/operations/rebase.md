# Rebase

Fetch latest and rebase onto base branch.

## Instructions

1. **Fetch latest from remote**: Run `git fetch origin`.

2. **Detect base branch**: Detect base branch per references/git-patterns.md.

3. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-patterns.md.

4. **Rebase onto base branch**: Run `gs upstack restack`. This rebases the current branch AND all branches above it onto their updated bases.

5. **If conflicts**: List conflicting files (`git diff --name-only --diff-filter=U`), report to user, present options via AskUserQuestion: "Help resolve conflicts", "Continue rebase (`gs rebase continue`)", or "Abort rebase (`gs rebase abort`)". After resolving conflicts manually, use `gs rebase continue` (alias `gs rbc`) instead of `git rebase --continue` — this resumes the rebase and auto-restacks upstack branches. Only run `gs rebase abort` (alias `gs rba`) if the user picks the abort option.

6. **If success**: Verify scope per references/git-patterns.md.

7. **Report**: Show commit count: `git rev-list --count origin/<base>..HEAD`.

