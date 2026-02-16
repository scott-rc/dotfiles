# Rerun Operation

Re-trigger failed CI jobs for the current branch.

## Instructions

1. **Find the failed run**:
   ```bash
   gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName
   ```
   If no failed runs exist, inform the user and stop.

2. **Rerun failed jobs**:
   ```bash
   gh run rerun <run-id> --failed
   ```
   - If `--failed` is not supported (older `gh` versions), fall back to:
     ```bash
     gh run rerun <run-id>
     ```

3. **Confirm rerun started**:
   ```bash
   gh run view <run-id> --json status
   ```
   MUST report the run ID and current status to the user.

4. **Report to user**:
   Confirm the rerun was triggered. Offer to run the check operation after a delay to monitor results.
