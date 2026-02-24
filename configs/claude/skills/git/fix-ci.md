# Fix CI Operation

Check CI status for the current branch. If failures exist, fetch logs, identify root cause, and fix the issues.

## Instructions

1. **Verify CI system**:
   ```bash
   test -d "$(git rev-parse --show-toplevel)/.github/workflows"
   ```
   If `.github/workflows/` does not exist, inform the user that no supported CI system was found and stop.

2. **Check CI status**:
   - If a PR exists (`gh pr view --json number,url 2>/dev/null`):
     ```bash
     gh pr checks
     ```
   - Otherwise check branch runs:
     ```bash
     gh run list --branch $(git branch --show-current) --limit 5
     ```
   Summarize: group checks by status (failed, pending, passed). If all pass, report success and stop. If pending, report which are still running and stop.

3. **Fetch failure logs**:
   ```bash
   gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName
   gh run view <run-id> --log-failed
   ```

4. **Parse logs and identify root cause**:
   If the log output is large (>200 lines), spawn a Task subagent (type: general-purpose, model: sonnet) to parse the logs. The subagent should extract only actionable information and return a concise failure summary. Otherwise, parse inline.

   MUST focus on actionable information:
   - Test failures: extract failing test names and assertion messages
   - Lint failures: extract file locations and specific errors
   - Build failures: extract compilation error messages
   - Focus on the first failure -- subsequent failures are often cascading
   - Truncate logs to the most relevant sections; do not dump raw output

5. **Load coding preferences**: MUST load the code skill (`skill: "code"`) for coding style preferences.

6. **Fix the issues**:
   - Read the relevant source files identified in the failure logs
   - Apply fixes for the root cause
   - If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options
   - If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user

7. **Verify fixes locally** (when possible):
   - Test failures: run the failing tests locally
   - Lint failures: run the linter on affected files
   - Build failures: run the build locally
   - If local verification is not possible (e.g., environment-specific failure), note this to the user

8. **Report to user**: Summarize what failed, why, what was fixed, and whether local verification passed. Offer to commit and push.
