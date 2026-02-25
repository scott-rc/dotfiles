# Watch Operation

Monitor the current PR for CI failures and new review comments. Triage failures, fix issues, commit, and push autonomously.

## Instructions

1. **Verify PR**:
   ```bash
   gh pr view --json number,url,headRefOid,state
   ```
   If no PR exists, inform the user and stop.

2. **Check for CI system**:
   ```bash
   test -d "$(git rev-parse --show-toplevel)/.github/workflows"
   ```
   If no `.github/workflows/`, note that CI monitoring is unavailable but continue -- review thread monitoring still works.

3. **Snapshot initial state**:
   - HEAD SHA: `git rev-parse HEAD`
   - Last push timestamp: `date -u +%Y-%m-%dT%H:%M:%SZ`
   - Unreplied threads via `get-pr-comments --unreplied` (path in [git-patterns.md](git-patterns.md))
   - CI status via `gh pr checks --json name,state,startedAt,completedAt` (if CI exists)
   - Any checks with `state` equal to `IN_PROGRESS`: add their IDs to `handled_runs` (avoids re-triaging reruns that are already underway)

   Initialize tracking state:
   - `handled_threads`: empty set of comment `id` values -- pre-existing unreplied threads are actionable and will be picked up on the first poll iteration
   - `handled_runs`: set of CI run database IDs already triaged (pre-seeded with in-progress runs)
   - `fix_attempts`: empty map of check name to attempt count
   - `head_sha`: current HEAD
   - `last_push_time`: current timestamp
   - `actions_log`: empty list for the final summary

4. **Report initial status**: CI check summary (pass/fail/pending counts), count of pre-existing unresolved threads, and that monitoring has started with 30s poll interval.

5. **Monitoring loop** (up to 90 iterations, ~45 minutes):

   > **All fixes in this loop MUST be delegated to subagents** -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path and an instruction to load the code skill (`skill: "code"`) for coding preferences.

   a. **Sleep**: `sleep 30`
      If any API call in the previous iteration returned HTTP 429, double the sleep interval (up to 120s). Reset to 30s once a non-429 response is received.

   b. **Poll CI** (if CI exists):
      ```bash
      gh pr checks --json name,state,startedAt,completedAt
      ```
      Only consider failures where `startedAt` is after `last_push_time`. Ignore stale checks from prior commits.

   c. **Poll review threads**:
      Run `get-pr-comments --unreplied` (path in [git-patterns.md](git-patterns.md)).
      New threads: any thread whose last comment `id` is NOT in `handled_threads`.

   d. **Handle new review threads** (if any):
      Delegate to a Task subagent (type: general-purpose, model: sonnet) with:
      - All new thread details: file path, line number, full comment bodies, last comment `id` per thread
      - Instruction: read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify

      After the subagent returns, add each thread's last comment `id` to `handled_threads` regardless of outcome (prevents re-dispatching on next poll).

      Check `git status --short`. If files changed:
      - Spawn the `committer` agent with prompt: "Commit these changes. They address PR review feedback: <brief summary of threads fixed>."
      - `git push`
      - Update `head_sha` and `last_push_time`
      - Reply to each fixed thread with a brief message referencing the fix commit SHA, per [pr-guidelines.md](pr-guidelines.md) ASCII rules. Use `gh api repos/{owner}/{repo}/pulls/{pr_number}/comments/{id}/replies -f body="<message>"`. Note: `{id}` must be the REST `databaseId` (from `get-pr-comments.sh`), not the GraphQL node ID.
      - Log to `actions_log`: threads fixed, files touched

      If the subagent reports it could not fix a thread, log it and continue -- do not block monitoring.

   e. **Handle CI failures** (if any new failures not in `handled_runs`):
      For each failed check:

      i. **Guard against infinite loops**: If `fix_attempts[check_name] >= 2`, log that repeated fixes have not resolved this check, skip it, and continue. Report to the user that manual intervention may be needed.

      ii. **Triage via ci-triager agent**:
         Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `ci-triager` agent with:
         - run_id: the failed run's database ID
         - workflow_name: the workflow name
         - branch: current branch
         - base_branch: detected base branch
         - repo: owner/repo string

         Based on the agent's classification:
         - **transient** or **flake**: add run ID to `handled_runs`, log the classification and indicator, continue.
         - **real**: proceed to step iii.

      iii. **Real failure -- fix it**:
         Delegate to a Task subagent (type: general-purpose, model: sonnet) with:
         - Trimmed logs and root cause from the ci-triager's report
         - Workflow name
         - Instruction: identify root cause, read relevant source files, fix the issue, run local verification if possible

         After the subagent returns, check `git status --short`. If files changed:
         - Spawn the `committer` agent with prompt: "Commit these changes. They fix a CI failure in <workflow>/<step>: <brief failure summary>."
         - `git push`
         - Update `head_sha` and `last_push_time`, increment `fix_attempts[check_name]`
         - Add run ID to `handled_runs`
         - Log to `actions_log`: what failed, what was fixed

         If the subagent reports it could not fix the issue, log it and continue.

   f. **Check exit conditions**:
      - **All green**: every check passes, none pending, and `get-pr-comments` returns zero unresolved threads -> report success, exit loop
      - **PR closed/merged**: `gh pr view --json state` shows MERGED or CLOSED -> report, exit loop
      - **Timeout**: max iterations reached -> report current status, exit loop

6. **Summary**: Report all actions from `actions_log`:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - CI jobs rerun (count, reasons: transient/flake)
   - Current CI status
   - Total commits pushed
