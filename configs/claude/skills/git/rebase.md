# Rebase Operation

Fetch the latest from the base branch and rebase the current branch onto it.

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

4. **If conflicts occur**:
   - List conflicting files: `git diff --name-only --diff-filter=U`
   - Report conflicts to the user
   - Offer to help resolve them or abort with `git rebase --abort`

5. **If rebase succeeds**: MUST verify branch scope following the scope verification pattern in [git-patterns.md](git-patterns.md). SHOULD also show commit count: `git rev-list --count origin/<base>..HEAD`

See [git-patterns.md](git-patterns.md) for base branch detection and scope verification patterns.
