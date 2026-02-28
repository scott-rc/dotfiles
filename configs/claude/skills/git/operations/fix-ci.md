# Fix CI

Fetch CI failure logs, triage via ci-triager, and fix the issues.

## Instructions

1. **Verify CI is configured**: Run `gh pr checks --json name,state 2>/dev/null` (or `gh run list --branch $(git branch --show-current) --limit 1` if no PR exists). If the command returns no check runs and no runs exist, inform the user that no CI checks were found and stop.

2. **Check CI status**:
   - If a PR exists (`gh pr view --json number,url 2>/dev/null`): run `gh pr checks`
   - Otherwise check branch runs: run `gh run list --branch $(git branch --show-current) --limit 5`
   - Summarize: group checks by status (failed, pending, passed). If all pass, report success and stop. If pending, report which are still running and stop.

3. **Triage via ci-triager agent**:
   Get all failed run IDs and workflow names using the `get-failed-runs` script path from references/git-patterns.md. `get-failed-runs` returns both runs with status=failure and in-progress runs that have at least one failed job; ci-triager uses its annotation fallback for the latter since full logs are unavailable until the run completes.
   If the script returns an empty array but step 2 found failures, the runs may still be initializing -- report to the user that CI is still in progress and suggest retrying shortly or using `/watch` to monitor automatically.
   Detect base branch per references/git-patterns.md. MUST use `ci-triager` for GitHub Actions. Spawn a `ci-triager` agent for each distinct failed run with: run_id, workflow_name, branch, base_branch, and repo.

   Based on the agent's classification:
   - **transient** or **flake**: report the classification to the user (the triager already reran the job). Stop.
   - **real**: proceed to step 4 using the trimmed logs and root cause from the triager's report.

4. **Fix the issues**:
   Dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch", forwarding the triager's full report as the task context.

   If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options before accepting the subagent's changes. If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user via AskUserQuestion before applying.

5. **Report to user**: Summarize what failed, why, what was fixed, and whether local verification passed. Offer to commit and push.
