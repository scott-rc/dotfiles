# Worktree

Create a new git worktree for the given task, or convert an existing branch into a worktree.

## Instructions

1. Parse the command arguments to determine the mode:
   - **Existing branch mode**: User explicitly specifies an existing branch name they want to convert to a worktree
   - **New branch mode**: User provides a task description to create a new branch

2. Determine the repository to use:
   - **If in the dotfiles repo** (repo path ends with `/dotfiles`): Treat this as "not in a git repo" and scan for other repositories instead
   - **If in a git repo** (not dotfiles): Use `git rev-parse --show-toplevel` to get the repo path
   - **If not in a git repo**: Scan `~/Code/*/` for git repositories (e.g., `~/Code/gadget/`, `~/Code/personal/`, `~/Code/scratch/`), excluding the dotfiles repo. If multiple repos found, try to infer the correct repo from the task description (e.g., if the task mentions "gadget" or a feature specific to a project, select that repo). If the repo cannot be inferred, ask the user to choose.

3. Get the repo name: `basename <repo-path>`

4. **For existing branch mode**:
   - Use the branch name as-is for the worktree directory name
   - Get the parent directory of the repo: `parent_dir=$(dirname <repo-path>)`
   - Run: `git worktree add "$parent_dir/<repo>-<branch-name>" <branch-name>`
   - Example directory structure after creating worktrees:
     ```
     ~/Code/gadget/
     ├── api/                    # main repo
     ├── api-fix-login-bug/      # worktree (sibling directory)
     └── api-add-auth/           # worktree (sibling directory)
     ```

5. **For new branch mode**:
   - Convert task description to kebab-case branch name (lowercase, hyphens for spaces, remove special chars)
   - Determine the base branch:
     - If user specifies "from <branch>", use that branch
     - Otherwise, use the current branch from the shell that invoked claude (shown in gitStatus at conversation start)
   - **Check for existing branch with same name**: `git show-ref --verify --quiet refs/heads/sc/<task-name>`
     - If the branch exists, check if it's an ancestor of the base branch (already merged): `git merge-base --is-ancestor sc/<task-name> <base-branch>`
       - If merged: delete the old branch first with `git branch -d sc/<task-name>`
       - If not merged: ask the user if they want to use the existing branch, delete it and start fresh, or use a different name
   - Get the parent directory of the repo: `parent_dir=$(dirname <repo-path>)`
   - Run: `git worktree add -b sc/<task-name> "$parent_dir/<repo>-<task-name>" <base-branch>`

6. Copy `.envrc.local` to the new worktree if it exists in the original repo

7. Run `direnv allow` in the new worktree directory to trust the environment:
   ```bash
   cd <new-worktree-path> && direnv allow
   ```

8. Set up `.vscode/settings.json` in the new worktree:
   - If `.vscode/settings.json` exists in the original repo:
     - Copy it to the new worktree (create `.vscode/` directory if needed)
     - Merge in the orange status bar customizations (add or update `workbench.colorCustomizations`)
   - If no `.vscode/settings.json` exists:
     - Create `.vscode/settings.json` with the orange status bar settings

   Orange status bar settings to add/merge:
   ```json
   {
     "workbench.colorCustomizations": {
       "statusBar.background": "#c75b39",
       "statusBar.foreground": "#ffffff",
       "statusBar.debuggingBackground": "#c75b39",
       "statusBar.noFolderBackground": "#c75b39"
     }
   }
   ```

9. Report the new worktree path and branch to the user
10. Ask if the user wants to cd into the worktree. If yes, use `fish -lc 'gw <dirname>'` where dirname is just the directory name (e.g., `api-fix-login`), not the full path.

See [git-patterns.md](git-patterns.md) for dotfiles exception pattern.
