# CI Operation

Check CI status and fetch failure logs to debug issues.

## Instructions

1. **Verify GitHub Actions is available**:
   ```bash
   ls -d .github/workflows 2>/dev/null
   ```
   - If `.github/workflows/` does not exist, inform the user that no supported CI system was found.

2. **Check if PR exists for current branch**:
   ```bash
   gh pr view --json number,url 2>/dev/null
   ```

3. **Get CI status**:

   If PR exists:
   ```bash
   gh pr checks
   ```

   If no PR (or for branch-level info):
   ```bash
   gh run list --branch $(git branch --show-current) --limit 5
   ```

4. **Summarize the status**:
   - Group by status (failed, pending, passed)
   - If all checks pass, report success and stop

5. **For failures, fetch detailed logs**:
   ```bash
   gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName
   ```
   ```bash
   gh run view <run-id> --log-failed
   ```

6. **Parse and present failure information**:
   - Test failures: failing test names and assertion errors
   - Lint failures: file locations and specific errors
   - Build failures: compilation errors
   - Keep output focused on actionable information
   - **Log truncation**: CI logs can be very long. Focus on:
     - The first failure in the output (subsequent failures are often cascading)
     - For test failures: show just the failing test name + assertion message
     - For build failures: show the error message, not full compilation output
     - Truncate logs to the most relevant sections

7. **Offer follow-up actions**:
   - For test failures: offer to look at failing tests
   - For lint failures: offer to fix lint errors
   - Always offer: re-run failed jobs with `gh run rerun <run-id> --failed`
