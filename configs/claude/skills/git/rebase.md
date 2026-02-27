# Rebase Operation

Fetch latest and rebase onto base branch.

## Instructions

1. **Fetch latest from remote**:
   ```bash
   git fetch origin
   ```

2. **Detect base branch**: Run `fish -c 'gbb'` to get the base branch name.

3. **Rebase onto base branch**:
   ```bash
   git rebase origin/<base>
   ```

4. **If conflicts**: List conflicting files (`git diff --name-only --diff-filter=U`), report to user, present options via AskUserQuestion: "Help resolve conflicts" or "Abort rebase (`git rebase --abort`)". Only run `git rebase --abort` if the user picks the abort option.

5. **If success**: Verify scope per [git-patterns.md](references/git-patterns.md), show commit count: `git rev-list --count origin/<base>..HEAD`.

See [git-patterns.md](references/git-patterns.md) for base branch detection and scope verification patterns.
