# Check CI

Check CI status for the current branch and report results.

## Instructions

1. **Verify CI is configured**: Run `gh pr checks --json name,state 2>/dev/null` (or `gh run list --branch $(git branch --show-current) --limit 1` if no PR exists). If the command returns no check runs and no runs exist, inform the user that no CI checks were found and stop.

2. **Check CI status**:
   - If a PR exists (`gh pr view --json number,url 2>/dev/null`): run `gh pr checks`
   - Otherwise check branch runs: run `gh run list --branch $(git branch --show-current) --limit 5`
   - Group checks by status (failed, pending, passed) and report a summary to the user.
