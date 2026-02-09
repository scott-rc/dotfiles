# Rebase Operation

Fetch the latest from the base branch and rebase the current branch onto it.

## Instructions

1. Fetch the latest from remote:
   ```bash
   git fetch origin
   ```

2. Detect the base branch:
   ```bash
   git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'
   ```

3. Rebase onto the base branch:
   ```bash
   git rebase origin/<base>
   ```

4. **If conflicts occur**:
   - List conflicting files: `git diff --name-only --diff-filter=U`
   - Report conflicts to the user
   - Offer to help resolve them or abort with `git rebase --abort`

5. **If rebase succeeds**, verify branch scope:
   - Show files changed: `git diff --name-only origin/<base> HEAD`
   - Show file stats: `git diff --stat origin/<base> HEAD`
   - Show commit count: `git rev-list --count origin/<base>..HEAD`
   - If unexpected files appear, warn the user:
     - This may indicate the rebase pulled in unrelated changes during conflict resolution
     - Offer to investigate with `git log --oneline origin/<base>..HEAD`
     - Offer to fix with `git rebase -i origin/<base>`

See [git-patterns.md](git-patterns.md) for base branch detection and scope verification patterns.
