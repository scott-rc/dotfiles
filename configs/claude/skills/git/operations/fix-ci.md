# Fix CI

Check CI status for the current branch. If failures exist, fetch logs, identify root cause, and fix the issues.

## Instructions

1. **Check invocation mode**: If invoked for status-only (e.g., "check CI", "CI status"), note this and stop after step 3 — report the current state without triaging or fixing failures.

2. **Verify CI is configured**: Run `gh pr checks --json name,state 2>/dev/null` (or `gh run list --branch $(git branch --show-current) --limit 1` if no PR exists). If the command returns no check runs and no runs exist, inform the user that no CI checks were found and stop.

3. **Check CI status**:
   - If a PR exists (`gh pr view --json number,url 2>/dev/null`): run `gh pr checks`
   - Otherwise check branch runs: run `gh run list --branch $(git branch --show-current) --limit 5`
   - Summarize: group checks by status (failed, pending, passed). If all pass, report success and stop. If pending, report which are still running and stop.
   - If invoked in status-only mode (step 1), stop here.

4. **Triage via ci-triager agent**:
   Get all failed run IDs and workflow names using the `get-failed-runs` script path from references/git-patterns.md.
   Detect base branch per references/git-patterns.md. SHOULD use `ci-triager` for GitHub Actions. Spawn a `ci-triager` agent for each distinct failed run with: run_id, workflow_name, branch, base_branch, and repo.

   Based on the agent's classification:
   - **transient** or **flake**: report the classification to the user (the triager already reran the job). Stop.
   - **real**: proceed to step 5 using the trimmed logs and root cause from the triager's report.

5. **Fix the issues**:
   Dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch", forwarding the triager's full report as the task context.

   If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options before accepting the subagent's changes. If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user via AskUserQuestion before applying.

6. **Report to user**: Summarize what failed, why, what was fixed, and whether local verification passed. Offer to commit and push.
