# Worktree

Create a new git worktree for the given task, or convert an existing branch into a worktree.

## Instructions

1. Parse the command arguments to determine the mode:
   - **Existing branch mode**: User specifies an existing branch name (check with `git show-ref --verify --quiet refs/heads/<name>`)
   - **New branch mode**: User provides a task description to create a new branch

2. Determine the repository to use:
   - **If in the dotfiles repo** (repo path ends with `/dotfiles`): Treat this as "not in a git repo" and scan for other repositories instead
   - **If in a git repo** (not dotfiles): Use `git rev-parse --show-toplevel` to get the repo path
   - **If not in a git repo**: Scan `~/Code/*/` for git repositories (e.g., `~/Code/gadget/`, `~/Code/personal/`, `~/Code/scratch/`), excluding the dotfiles repo. If multiple repos found, try to infer the correct repo from the task description (e.g., if the task mentions "gadget" or a feature specific to a project, select that repo). If the repo cannot be inferred, ask the user to choose.

3. Get the repo name: `basename <repo-path>`

4. **For existing branch mode**:
   - Use the branch name as-is for the worktree directory name
   - Run: `git worktree add ../<repo>-<branch-name> <branch-name>`
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
   - Run: `git worktree add -b sc/<task-name> ../<repo>-<task-name> <base-branch>`

6. Copy `.envrc.local` to the new worktree if it exists in the original repo

7. Create `.vscode/settings.json` in the new worktree with a distinct orange status bar:
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

8. Report the new worktree path and branch to the user
9. Ask if the user wants to open the worktree in Cursor (use `fish -lc 'gw <dirname>'` to open, where dirname is just the directory name, not the full path). The `gw` command handles `direnv allow` automatically.
