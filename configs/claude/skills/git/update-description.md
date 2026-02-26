# Update Description

Rewrite the PR title and description to match current changes per guidelines.

## Instructions

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.
2. **Delegate to `pr-writer` agent** with: mode `update`, base_branch (detect per [git-patterns.md](git-patterns.md)), pr_number. Pass only the documented input parameters — do NOT include formatting rules or style guidance (the agent owns its own rules). If the agent fails, re-spawn it once — if it fails again, report the error to the user. Do NOT write the PR description yourself.
3. **Report**: confirm update, show PR URL.
