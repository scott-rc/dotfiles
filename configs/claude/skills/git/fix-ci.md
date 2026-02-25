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

3. **Triage via ci-triager agent**:
   Get all failed run IDs and workflow names:
   ```bash
   gh run list --branch $(git branch --show-current) --status failure --limit 5 --json databaseId,workflowName
   ```
   Detect base branch per [git-patterns.md](git-patterns.md). Spawn a `ci-triager` agent for each distinct failed run with: run_id, workflow_name, branch, base_branch, and repo.

   Based on the agent's classification:
   - **transient** or **flake**: report the classification to the user (the triager already reran the job). Stop.
   - **real**: proceed to step 4 using the trimmed logs and root cause from the triager's report.

4. **Load coding preferences**: MUST load the code skill (`skill: "code"`) for coding style preferences.

5. **Fix the issues**:
   - Read the relevant source files identified in the triager's logs
   - Apply fixes for the root cause
   - If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options
   - If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user

6. **Verify fixes locally** (when possible):
   - Test failures: run the failing tests locally
   - Lint failures: run the linter on affected files
   - Build failures: run the build locally
   - If local verification is not possible (e.g., environment-specific failure), note this to the user

7. **Report to user**: Summarize what failed, why, what was fixed, and whether local verification passed. Offer to commit and push.
