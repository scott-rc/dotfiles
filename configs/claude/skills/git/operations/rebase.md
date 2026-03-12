# Rebase

Fetch latest and rebase onto base branch.

## Instructions

1. **Fetch latest from remote**: Run `git fetch origin`.

2. **Detect base branch**: Detect base branch per references/git-patterns.md.

3. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-patterns.md.

4. **Rebase onto base branch**: Run `gs upstack restack`. This rebases the current branch AND all branches above it onto their updated bases.

5. **If conflicts**: List conflicting files (`git diff --name-only --diff-filter=U`), report to user, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git rebase --abort`)". Only run `git rebase --abort` if the user picks the abort option.

6. **If success**: Verify scope per references/git-patterns.md.

7. **Report**: Show commit count: `git rev-list --count origin/<base>..HEAD`.

