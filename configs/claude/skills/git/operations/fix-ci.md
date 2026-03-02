# Fix CI

Fetch CI failure logs, triage via ci-triager, and fix the issues.

## Instructions

1. **Detect CI state**: Follow the "CI Detection" pattern in references/git-patterns.md (steps 1 and 2). Group checks by status (failed, pending, passed). If all pass, report success and stop. If pending, report which are still running and stop.
   Detect the CI system per the "CI System Detection" section in references/git-patterns.md.
   - If Buildkite: skip step 2 and go directly to step 3 (see references/buildkite-handling.md).
   - If GitHub Actions: continue to step 2.

2. **Triage via ci-triager agent** (GitHub Actions only):
   Get all failed run IDs and workflow names using the `get-failed-runs` script (path in references/git-patterns.md).
   If the script returns an empty array but step 1 found failures, runs may still be initializing. Report to the user that CI is still in progress and suggest retrying shortly.
   Detect base branch per references/git-patterns.md. Spawn a `ci-triager` agent for each distinct failed run with: run_id, workflow_name, branch, base_branch, and repo.

   Based on the agent's classification:
   - **transient** or **flake**: report the classification to the user (the triager already reran the job). Stop.
   - **real**: proceed to step 3 using the trimmed logs and root cause from the triager's report.

3. **Fix the issues**:
   For Buildkite: fetch logs per references/buildkite-handling.md.
   For GitHub Actions: forward the triager's full report as task context.
   Dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch".

   If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options before accepting the subagent's changes. If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user via AskUserQuestion before applying.

4. **Report to user**: Summarize what failed, why, what was fixed, and whether local verification passed. Offer to commit and push — if the user accepts, dispatch the `committer` agent to commit, then run the Push operation (push's uncommitted-changes check is redundant after a fresh commit — skip it).
