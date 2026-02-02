# Worktree

Create a new git worktree for the given task, or convert an existing branch into a worktree.

## Instructions

1. Parse the command arguments to determine the mode:
   - **Existing branch mode**: User specifies an existing branch name (check with `git show-ref --verify --quiet refs/heads/<name>`)
   - **New branch mode**: User provides a task description to create a new branch

2. Get the current repo name: `basename $(git rev-parse --show-toplevel)`

3. **For existing branch mode**:
   - Use the branch name as-is for the worktree directory name
   - Run: `git worktree add ../<repo>-<branch-name> <branch-name>`

4. **For new branch mode**:
   - Convert task description to kebab-case branch name (lowercase, hyphens for spaces, remove special chars)
   - Determine the base branch:
     - If user specifies "from <branch>", use that branch
     - If context suggests a specific branch, use that
     - Otherwise, detect default: `git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'`
   - Run: `git worktree add -b sc/<task-name> ../<repo>-<task-name> <base-branch>`

5. Create `.vscode/settings.json` in the new worktree with a distinct orange status bar:
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

6. Report the new worktree path and branch to the user
7. Ask if the user wants to open the worktree in Cursor (via `gw`)
