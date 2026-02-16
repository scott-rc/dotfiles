# Check Operation

Check CI status for the current branch and summarize results.

## Instructions

1. **Verify CI system**:
   ```bash
   test -d "$(git rev-parse --show-toplevel)/.github/workflows"
   ```
   If `.github/workflows/` does not exist, inform the user that no supported CI system was found and stop.

2. **Check for PR**:
   ```bash
   gh pr view --json number,url 2>/dev/null
   ```

3. **Get CI status**:
   - If PR exists:
     ```bash
     gh pr checks
     ```
   - If no PR (or for branch-level runs):
     ```bash
     gh run list --branch $(git branch --show-current) --limit 5
     ```

4. **Summarize results**:
   - Group checks by status: failed, pending, passed
   - If all checks pass, report success
   - If failures exist, list the failing check names
   - If checks are pending, report which ones are still running

5. **Report to user**:
   MUST present a concise summary. For failures, offer to run the diagnose operation for details.
