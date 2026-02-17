# Diagnose Operation

Fetch CI failure logs, identify root cause, and fix the issues.

## Instructions

1. **Get failed run**:
   ```bash
   gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName
   ```
   If no failed runs exist, run the check operation to get current status and stop.

2. **Fetch failure logs**:
   ```bash
   gh run view <run-id> --log-failed
   ```

3. **Parse logs and identify root cause**:
   MUST focus on actionable information and apply these rules:
   - Test failures: extract failing test names and assertion messages
   - Lint failures: extract file locations and specific errors
   - Build failures: extract compilation error messages
   - Focus on the first failure in the output — subsequent failures are often cascading
   - Truncate logs to the most relevant sections; do not dump raw output

4. **Load coding preferences**: MUST read [general-guidelines.md](../code/general-guidelines.md). If fixing TypeScript/JavaScript files, also read [typescript-guidelines.md](../code/typescript-guidelines.md). Apply these preferences when writing fixes.

5. **Fix the issues**:
   - Read the relevant source files identified in the failure logs
   - Apply fixes for the root cause
   - If the fix is ambiguous or risky, present options to the user before proceeding
   - If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user

6. **Verify fixes locally** (when possible):
   - For test failures: run the failing tests locally
   - For lint failures: run the linter on affected files
   - For build failures: run the build locally
   - If local verification is not possible (e.g., environment-specific failure), note this to the user

7. **Report to user**:
   MUST summarize: what failed, why, what was fixed, and whether local verification passed. Offer to commit the fixes and rerun CI.
