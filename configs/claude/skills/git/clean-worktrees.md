# Clean Operation

Remove merged, squash-merged, and orphaned worktrees using the `gwc` fish function.

## Instructions

1. **Determine repositories to scan**:
   - **If in a git repo**: Use only the current repository (unless it's the dotfiles repo)
   - **If in the dotfiles repo** (repo path ends with `/dotfiles`): Treat as "not in a git repo" and scan other repositories
   - **If not in a git repo**: Scan all repositories in `~/Code/*/*`, excluding the dotfiles repo

2. **Fetch and prune** for each repository: `git -C <repo> fetch origin --prune --quiet`

3. **Discover stale worktrees** by running `fish -c 'gwc'` (or `fish -c 'gwc <repo>/.worktrees'` for multi-repo). The function's `read` prompt receives no input from the Bash tool, so it lists stale entries without deleting. `gwc` detects three kinds of stale worktrees:
   - **Orphaned**: directories in `.worktrees/` not tracked by `git worktree list`
   - **Merged**: branch is ancestor of `origin/<default-branch>`
   - **Gone**: upstream tracking ref deleted (squash-merged)

4. **Present stale worktrees** to the user with their labels. If none found, report that and stop.

5. **Confirm deletion**: MUST confirm with the user before removing.

6. **Delete** by piping confirmation: `fish -c 'printf y\n | gwc <repo>/.worktrees'`

7. **Prune stale references**: `git -C <repo> worktree prune`

8. **Report summary** of what was cleaned up across all repositories.

See [git-patterns.md](git-patterns.md) for dotfiles exception pattern.
