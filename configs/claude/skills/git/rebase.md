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

4. If the rebase succeeds, report success to the user.

5. If there are conflicts:
   - Report the conflicting files
   - Offer to help resolve them or abort with `git rebase --abort`
