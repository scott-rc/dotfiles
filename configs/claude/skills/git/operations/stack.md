# Stack

Navigate and manage stacked branches tracked by git-spice.

## Instructions

1. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-spice-patterns.md. This auto-initializes silently if needed.

2. **Route by intent**:
   - **Navigate**: `git-spice up`, `git-spice down`, `git-spice top`, `git-spice bottom`, `git-spice trunk` — move to the requested position in the stack.
   - **Restack**: `git-spice upstack restack` (current branch and all above) — rebase the stack onto updated bases.
   - **List**: `git-spice log short` — show the stack with current position and CR status.
   - **Track**: `git-spice branch track` — add the current branch to git-spice tracking.
   - **Untrack**: `git-spice branch untrack` — remove the current branch from tracking.
   - **Reorder**: Move a branch to a new position in the stack. Parse the user's request to identify which branch to move and where. If ambiguous (e.g., "rebase X after Y" could mean multiple things), ask via AskUserQuestion to clarify which branch moves and which is the target base. Show the current stack topology: `git-spice log short`. Run the Downstream PR Safety check from references/git-patterns.md — reordering changes branch bases, which may affect open PRs; do this BEFORE executing the move. Present the before/after topology and confirm via AskUserQuestion before executing. Execute using the appropriate git-spice command: for moving the branch and all branches above it use `git-spice upstack onto <destination> --no-prompt`; for moving only the current branch use `git-spice branch onto <destination> --no-prompt`; if the user requests reordering via interactive editor (`git-spice stack edit`), inform them that `git-spice stack edit` requires an interactive terminal and cannot run in an agent context — suggest they run it manually, then re-run `/git stack list` after completion. Show the resulting stack: `git-spice log short`.
   - **Fold**: Merge the current branch into its base and remove it from the stack. MUST warn the user that this deletes the branch and confirm via AskUserQuestion before executing. Run the Branch Fold pattern from references/git-spice-patterns.md — it handles downstream PR base migration (critical: prevents GitHub from auto-closing PRs), fold execution, and cleanup.
   - **Delete**: Confirm via AskUserQuestion before executing — this is destructive and irreversible. For a single branch: `git-spice branch delete --force --no-prompt`. For the entire stack: `git-spice stack delete --force --no-prompt`.
   - **Diff**: `git-spice branch diff` — show the diff between the current branch and its base. Quick way to see what the current branch changes.
   - **Branch squash**: `git-spice branch squash --no-prompt` — squash all commits in the current branch into one and auto-restack upstack branches. Use `-m "<msg>"` to provide the message inline, or `--no-edit` to keep the combined message without opening an editor.

3. **Report**: Show current position and stack state via `git-spice log short`.
