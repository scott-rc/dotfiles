# Sync

Fetch latest, clean up merged branches, and restack.

## Instructions

1. **Check git-spice initialization**: Detect per references/git-patterns.md (Git-Spice > Detection).
   - If initialized: run `gs repo sync --restack --no-prompt`. This fetches from the remote, deletes branches that have been merged, and restacks remaining branches.
   - If not initialized: Sync does not auto-initialize git-spice (only Split and Stack do). Fall back to `git fetch origin` followed by `git rebase origin/<base>` (detect base per references/git-patterns.md Base Branch Detection).

2. **Report**: Summarize what happened — branches synced, merged branches cleaned up, restacked branches. If using the git-spice path, show `gs log short` for the current stack state.
