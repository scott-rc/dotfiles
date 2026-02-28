# Rerun

Re-trigger failed CI jobs on the current branch.

## Instructions

1. **Find failed run**: Run `gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName`. If no failed run is found, inform the user and stop.

2. **Rerun**: Run `gh run rerun <run-id> --failed`. If that command is unsupported, fall back to `gh run rerun <run-id>`.

3. **Check new status**: Run `gh run view <run-id> --json status`. Report the run ID and status. If status is still `failed` (not `queued` or `in_progress`), report the error to the user.

4. **Offer to watch**: Offer to run the Watch operation to monitor results.
