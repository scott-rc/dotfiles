# Worktree

Create a new git worktree for the given task.

## Instructions

1. Parse the task description from the command arguments
2. Convert to kebab-case branch name (lowercase, hyphens for spaces, remove special chars)
3. Get the current repo name: `basename $(git rev-parse --show-toplevel)`
4. Determine the base branch:
   - If user specifies "from <branch>", use that branch
   - If context suggests a specific branch, use that
   - Otherwise, detect default: `git rev-parse --abbrev-ref origin/HEAD 2>/dev/null | sed 's|origin/||' || echo 'main'`
5. Run: `git worktree add -b sc/<task-name> ../<repo>-<task-name> <base-branch>`
6. Create `.vscode/settings.json` in the new worktree with a distinct orange status bar:
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
7. Report the new worktree path and base branch to the user
8. Ask if the user wants to open the worktree in Cursor (via `gw`)
