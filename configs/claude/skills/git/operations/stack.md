# Stack

Navigate and manage stacked branches tracked by git-spice.

## Instructions

1. **Check git-spice initialization**: Detect per references/git-patterns.md (Git-Spice > Detection). If not initialized, ask via AskUserQuestion: "This repo isn't initialized for git-spice. Initialize it?" with options: "Initialize" or "Cancel". If Initialize, run initialization per references/git-patterns.md (Git-Spice > Initialization) using the base branch from Base Branch Detection.

2. **Route by intent**:
   - **Navigate**: `gs up`, `gs down`, `gs top`, `gs bottom`, `gs trunk` — move to the requested position in the stack.
   - **Restack**: `gs upstack restack` (current branch and all above) — rebase the stack onto updated bases.
   - **List**: `gs log short` — show the stack with current position and CR status.
   - **Track**: `gs branch track` — add the current branch to git-spice tracking.
   - **Untrack**: `gs branch untrack` — remove the current branch from tracking.

3. **Report**: Show current position and stack state via `gs log short`.
