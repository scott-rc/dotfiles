# Rebase Operation

Fetch the latest from the base branch and rebase the current branch onto it.

## Instructions

1. Fetch the latest from remote:
   ```bash
   git fetch origin
   ```
   > **Note**: Never use `git fetch origin <branch>:<branch>` - this fails if the branch is checked out in another worktree. Always use `git fetch origin` and reference `origin/<branch>`.

2. Detect the base branch:
   ```bash
   git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'
   ```

3. Rebase onto the base branch:
   ```bash
   git rebase origin/<base>
   ```

4. If the rebase succeeds, **verify branch scope**:
   - Get files changed: `git diff --name-only origin/<base> HEAD`
   - Get commit count: `git rev-list --count origin/<base>..HEAD`
   - Show this summary to the user for verification
   - If the file list contains unexpected files (files not related to the branch's purpose), warn the user:
     - This may indicate the rebase pulled in unrelated changes during conflict resolution
     - Offer to help investigate or abort with `git rebase --abort` (only works if rebase just completed and no new operations done)

5. If there are conflicts:
   - Report the conflicting files
   - Offer to help resolve them or abort with `git rebase --abort`
