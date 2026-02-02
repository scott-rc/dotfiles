# Clean Worktrees

Remove git worktrees whose branches have already been merged to main.

## Instructions

1. Determine which repositories to scan:
   - **If in a git repo**: Use only the current repository (unless it's the dotfiles repo)
   - **If in the dotfiles repo** (repo path ends with `/dotfiles`): Treat as "not in a git repo" and scan other repositories
   - **If not in a git repo**: Scan all repositories in `~/Code/*/*`, excluding the dotfiles repo

2. For each repository, list all worktrees using `git worktree list --porcelain`

3. For each worktree (excluding the main worktree):
   - Get the branch name from the worktree
   - Check if the branch has been merged to main/master:
     - First determine default branch: `git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'` (fallback to "main")
     - Fetch latest: `git fetch origin <default-branch> --quiet`
     - Check if merged: `git branch --merged origin/<default-branch>` and see if the worktree's branch is in the list

4. Present the list of merged worktrees to the user:
   - Show the worktree path and branch name for each
   - If no merged worktrees found, report that and exit

5. Ask the user to confirm deletion (show the full list)

6. For each confirmed worktree:
   - Run `git worktree remove <path>` to remove the worktree
   - If the branch still exists, offer to delete it with `git branch -d <branch>`

7. Run `git worktree prune` to clean up any stale worktree references

8. Report summary of what was cleaned up
