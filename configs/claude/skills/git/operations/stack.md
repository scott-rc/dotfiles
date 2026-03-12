# Stack

Navigate and manage stacked branches tracked by git-spice.

## Instructions

1. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-patterns.md. This auto-initializes silently if needed.

2. **Route by intent**:
   - **Navigate**: `gs up`, `gs down`, `gs top`, `gs bottom`, `gs trunk` — move to the requested position in the stack.
   - **Restack**: `gs upstack restack` (current branch and all above) — rebase the stack onto updated bases.
   - **List**: `gs log short` — show the stack with current position and CR status.
   - **Track**: `gs branch track` — add the current branch to git-spice tracking.
   - **Untrack**: `gs branch untrack` — remove the current branch from tracking.
   - **Reorder**: Move a branch to a new position in the stack. Parse the user's request to identify which branch to move and where. If ambiguous (e.g., "rebase X after Y" could mean multiple things), ask via AskUserQuestion to clarify which branch moves and which is the target base. Show the current stack topology: `gs log short`. Run the Downstream PR Safety check from references/git-patterns.md — reordering changes branch bases, which may affect open PRs; do this BEFORE executing the move. Present the before/after topology and confirm via AskUserQuestion before executing. Execute using the appropriate git-spice command from references/git-patterns.md (Git-Spice > Branch Reorder): for moving the branch and all branches above it use `gs upstack onto <destination> --no-prompt`; for moving only the current branch use `gs branch onto <destination> --no-prompt`; if the user requests reordering via interactive editor (`gs stack edit`), inform them that `gs stack edit` requires an interactive terminal and cannot run in an agent context — suggest they run it manually, then re-run `/git stack list` after completion. Show the resulting stack: `gs log short`.
   - **Fold**: `gs branch fold --no-prompt` — merges the current branch into its base branch and deletes the current branch; upstack branches rebase onto the next downstack branch. MUST warn the user that this deletes the branch and confirm via AskUserQuestion before executing.
   - **Delete**: Confirm via AskUserQuestion before executing — this is destructive and irreversible. For a single branch: `gs branch delete --force --no-prompt`. For the entire stack: `gs stack delete --force --no-prompt`.
   - **Diff**: `gs branch diff` — show the diff between the current branch and its base. Quick way to see what the current branch changes.
   - **Branch squash**: `gs branch squash --no-prompt` — squash all commits in the current branch into one and auto-restack upstack branches. Use `-m "<msg>"` to provide the message inline, or `--no-edit` to keep the combined message without opening an editor.

3. **Report**: Show current position and stack state via `gs log short`.
