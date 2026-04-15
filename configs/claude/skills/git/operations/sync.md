# Sync

Fetch latest, clean up merged branches, and restack.

## Instructions

1. **Sync**: Run `git-spice repo sync --restack --no-prompt`. This fetches from the remote, deletes branches that have been merged, and restacks remaining branches.

2. **Report**: Summarize what happened — branches synced, merged branches cleaned up, restacked branches. Show `git-spice log short` for the current stack state.
