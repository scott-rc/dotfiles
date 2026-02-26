# Update Description

Rewrite the PR title and description to match current changes per guidelines.

## Instructions

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.
2. **Delegate to `pr-writer` agent** with: mode `update`, base_branch (detect per [git-patterns.md](git-patterns.md)), pr_number.
3. **Report**: confirm update, show PR URL.
