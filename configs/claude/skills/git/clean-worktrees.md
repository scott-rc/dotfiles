# Clean Operation

Remove worktrees whose branches have been merged to main.

## Instructions

1. **Determine repositories to scan**:
   - **If in a git repo**: Use only the current repository (unless it's the dotfiles repo)
   - **If in the dotfiles repo** (repo path ends with `/dotfiles`): Treat as "not in a git repo" and scan other repositories
   - **If not in a git repo**: Scan all repositories in `~/Code/*/*`, excluding the dotfiles repo

2. **List worktrees** for each repository using `git worktree list --porcelain`

3. **Check merge status** for each worktree (excluding the main worktree):
   - Get the branch name from the worktree
   - Detect the base branch (see [git-patterns.md](git-patterns.md))
   - Fetch and prune: `git fetch origin --prune --quiet`
   - Check if the branch should be cleaned up (either condition):
     - **Merged**: `git branch --merged origin/<default-branch>` includes the branch
     - **Squash-merged**: Remote branch was deleted (check with `git ls-remote --heads origin <branch>` returning empty)

4. **Present merged worktrees** to the user:
   - Show the worktree path and branch name for each
   - If no merged worktrees found, report that and exit

5. **Confirm deletion** with the user (show the full list)

6. **Remove confirmed worktrees**:
   - Run `git worktree remove <path>` to remove the worktree
   - Delete the local branch: use `git branch -d <branch>`, or `git branch -D <branch>` for squash-merged branches

7. **Prune stale references**: `git worktree prune`

8. **Report summary** of what was cleaned up

See [git-patterns.md](git-patterns.md) for base branch detection and dotfiles exception patterns.
