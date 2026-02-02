# CI Operation

Check CI status and fetch failure logs to debug issues.

## Instructions

1. **Detect the CI system**:
   ```bash
   ls -d .github/workflows .buildkite 2>/dev/null
   ```
   - If `.github/workflows/` exists → GitHub Actions
   - If `.buildkite/` exists → Buildkite
   - If neither or only Buildkite: inform user that only GitHub Actions is currently supported

2. **For GitHub Actions repos only**, proceed with the following steps.

### GitHub Actions

3. **Check if PR exists for current branch**:
   ```bash
   gh pr view --json number,url 2>/dev/null
   ```

4. **Get CI status**:

   If PR exists:
   ```bash
   gh pr checks
   ```

   If no PR (or for branch-level info):
   ```bash
   gh run list --branch $(git branch --show-current) --limit 5
   ```

5. **Summarize the status**:
   - Group by status (failed, pending, passed)
   - If all checks pass, report success and stop

6. **For failures, fetch detailed logs**:
   ```bash
   gh run list --branch $(git branch --show-current) --status failure --limit 1 --json databaseId,workflowName
   ```
   ```bash
   gh run view <run-id> --log-failed
   ```

7. **Parse and present failure information**:
   - Test failures: failing test names and assertion errors
   - Lint failures: file locations and specific errors
   - Build failures: compilation errors
   - Keep output focused on actionable information

8. **Offer follow-up actions**:
   - For test failures: offer to look at failing tests
   - For lint failures: offer to fix lint errors
   - Always offer: re-run failed jobs with `gh run rerun <run-id> --failed`
