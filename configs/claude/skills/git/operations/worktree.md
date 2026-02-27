# Worktree

Create a new git worktree for the given task, or convert an existing branch into a worktree. Delegates the mechanical work to the `gwt` fish function.

## Instructions

1. **Determine repository**:
   - **If in the dotfiles repo** (repo path ends with `/dotfiles`): Treat this as "not in a git repo" and scan for other repositories instead
   - **If in a git repo** (not dotfiles): Use `git rev-parse --show-toplevel` to get the repo path
   - **If not in a git repo**: List repositories with `ls -d ~/Code/*/*/.git 2>/dev/null | sed 's|/.git$||'` (excluding dotfiles). If multiple repos found, try to infer from the task description. If ambiguous, present matching repos as AskUserQuestion options.

2. **Determine mode** from the user's request:
   - **Existing branch**: User names a specific branch to check out as a worktree
   - **New branch**: User describes a task (words become the branch name)

3. **Determine base branch** (new branch mode only):
   - If user specifies "from <branch>", use that
   - Otherwise, use the current branch from gitStatus at conversation start

4. **Generate branch slug** (new branch mode only):
   - Generate a concise 2-4 word kebab-case slug
   - Pass the slug words as positional args to `gwt` (it will kebab-case them)
   - `gwt` automatically prefixes the branch with `sc/` -- do NOT add the prefix yourself
   - Example: task "fix the login page redirect bug" → args: `fix login redirect` → branch: `sc/fix-login-redirect`

5. **Run `gwt`** via `fish -c '...'` with the appropriate flags:
   - Repo outside cwd: `-C <repo-path>`
   - Existing branch: `--branch <name>`
   - New branch: slug words as positional args, plus `--from <base>` if not current branch
   - Example: `fish -c 'gwt -C ~/Code/gadget/gadget --from main fix login redirect'`

6. **Handle exit codes**:
   - **0**: Success -- proceed to step 7
   - **1**: Error -- report the output to the user
   - **2**: Branch exists and is not merged -- present options via AskUserQuestion: "Use existing branch", "Delete and start fresh", "Choose a different name"

7. **Report success**: Tell the user the `cd` command is on their clipboard and exit Claude so they can paste it.
