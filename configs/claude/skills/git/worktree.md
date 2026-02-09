# Worktree Operation

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
   - Run: `git worktree add <repo-path>/.worktrees/<branch-name> <branch-name>`
   - Example directory structure after creating worktrees:
     ```
     ~/Code/gadget/
     └── api/                       # main repo
         └── .worktrees/
             ├── fix-login-bug/     # worktree
             └── add-auth/          # worktree
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
   - Run: `git worktree add -b sc/<task-name> <repo-path>/.worktrees/<task-name> <base-branch>`

6. Copy `.envrc.local` to the new worktree if it exists in the original repo

7. Copy additional local configuration files to the new worktree (if they exist in the original repo):
   - `.env.local` - local environment variables
   - `CLAUDE.local.md` - local Claude instructions
   - `.mcp.json` - local MCP configuration
   - `.claude/**/*.local.*` files (e.g., `.claude/settings.local.json`, `.claude/hooks/presubmit.local.sh`)
     - Create the `.claude/` directory structure as needed
     - Preserve the directory structure when copying (e.g., `.claude/hooks/foo.local.sh` → `.claude/hooks/foo.local.sh`)

8. Run `direnv allow` in the new worktree directory to trust the environment:
   ```bash
   cd <new-worktree-path> && direnv allow
   ```

9. Report the new worktree path and branch to the user
10. Copy the cd command to the clipboard and print it as a fallback:
    ```bash
    echo "cd <new-worktree-path>" | pbcopy
    ```
    Print the command as well:
    ```
    cd <new-worktree-path>
    ```
    Tell the user the command is in their clipboard, then exit Claude so they can paste and run it.

See [git-patterns.md](git-patterns.md) for dotfiles exception pattern.
