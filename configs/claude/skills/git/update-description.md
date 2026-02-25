# Update Description Operation

Rewrite the PR title and description to match the current changes per guidelines.

## Instructions

1. **Check for existing PR**:
   ```bash
   gh pr view --json number,url,title,body 2>/dev/null
   ```
   If no PR exists, inform the user and stop.

2. **Update via pr-writer agent**:
   Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `pr-writer` agent with: mode `update`, base_branch, pr_number.

3. **Report to user**: Confirm the PR was updated and show the PR URL.
