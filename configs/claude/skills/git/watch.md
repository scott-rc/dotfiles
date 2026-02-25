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
   - Unreplied threads via `~/.claude/skills/git/scripts/get-pr-comments.sh --unreplied`
   - CI status via `gh pr checks --json name,status,conclusion,startedAt` (if CI exists)
   - Any runs currently `in_progress`: add their IDs to `handled_runs` (avoids re-triaging reruns that are already underway)

   Initialize tracking state:
   - `handled_threads`: empty set of comment `id` values -- pre-existing unreplied threads are actionable and will be picked up on the first poll iteration
   - `handled_runs`: set of CI run database IDs already triaged (pre-seeded with in-progress runs)
   - `fix_attempts`: empty map of check name to attempt count
   - `head_sha`: current HEAD
   - `last_push_time`: current timestamp
   - `actions_log`: empty list for the final summary

4. **Report initial status**: CI check summary (pass/fail/pending counts), count of pre-existing unresolved threads, and that monitoring has started with 30s poll interval.

5. **Monitoring loop** (up to 90 iterations, ~45 minutes):

   a. **Sleep**: `sleep 30`
      If any API call in the previous iteration returned HTTP 429, double the sleep interval (up to 120s). Reset to 30s once a non-429 response is received.

   b. **Poll CI** (if CI exists):
      ```bash
      gh pr checks --json name,status,conclusion,startedAt
      ```
      Only consider failures where `startedAt` is after `last_push_time`. Ignore stale checks from prior commits.

   c. **Poll review threads**:
      ```bash
      ~/.claude/skills/git/scripts/get-pr-comments.sh --unreplied
      ```
      New threads: any thread whose last comment `id` is NOT in `handled_threads`.

   d. **Handle new review threads** (if any):
      MUST delegate fixes to a Task subagent (type: general-purpose, model: sonnet). The watch loop is long-running -- reading source files and attempting fixes inline exhausts the context window. The orchestrator triages; the subagent debugs and fixes.

      Subagent prompt MUST include:
      - All new thread details: file path, line number, full comment bodies, last comment `id` per thread
      - Repository root path
      - Instruction: read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify. MUST load the code skill (`skill: "code"`) for coding preferences.

      MUST NOT read source files, explore the codebase, or attempt fixes inline -- all code changes happen in the subagent.

      After the subagent returns, check `git status --short`. If files changed:
      - Commit per [commit-guidelines.md](commit-guidelines.md), referencing the review feedback
      - `git push`
      - Update `head_sha` and `last_push_time`
      - Add each thread's last comment `id` to `handled_threads`
      - Reply to each fixed thread with a brief message referencing the fix commit SHA, per [pr-guidelines.md](pr-guidelines.md) ASCII rules. Use `gh api repos/{owner}/{repo}/pulls/comments/{id}/replies -f body="<message>"`.
      - Log to `actions_log`: threads fixed, files touched

      If the subagent reports it could not fix a thread, log it and continue -- do not block monitoring.

   e. **Handle CI failures** (if any new failures not in `handled_runs`):
      For each failed check:

      i. **Guard against infinite loops**: If `fix_attempts[check_name] >= 2`, log that repeated fixes have not resolved this check, skip it, and continue. Report to the user that manual intervention may be needed.

      ii. **Fetch failure logs**:
         ```bash
         gh run list --branch $(git branch --show-current) --status failure --limit 5 --json databaseId,workflowName
         gh run view <run-id> --log-failed 2>&1 | tail -300
         ```

      iii. **Triage -- transient/infrastructure?**
           Scan logs for indicators:
           - timeout, ETIMEDOUT
           - connection refused, ECONNREFUSED
           - rate limit, 429
           - 503, 502, 504
           - OOM, out of memory
           - killed, signal 9
           - runner lost
           - no space left on device
           - could not resolve host
           - socket hang up

           If found: `gh run rerun <id> --failed`, add to `handled_runs`, log as transient rerun, continue.

      iv. **Triage -- flake?**
          Detect base branch per [git-patterns.md](git-patterns.md), then check if the same workflow has failed on the base branch recently:
          ```bash
          gh run list --branch <base> --status failure --limit 5 --json databaseId,workflowName,createdAt
          ```
          If the same workflow name appears with a failure in the last 7 days: `gh run rerun <id> --failed`, add to `handled_runs`, log as flake rerun, continue.

      v. **Real failure -- fix it**:
         MUST delegate the fix to a Task subagent (type: general-purpose, model: sonnet). The watch loop is long-running -- reading source files, analyzing code, and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages; the subagent debugs and fixes.

         Subagent prompt MUST include:
         - Failure logs (relevant sections, not raw dump)
         - Workflow name and failing step
         - Repository root path
         - Instruction: identify root cause, read relevant source files, fix the issue, run local verification if possible. MUST load the code skill (`skill: "code"`) for coding preferences.

         MUST NOT read source files, explore the codebase, or attempt fixes inline -- all debugging and code changes happen in the subagent.

         After the subagent returns, check `git status --short`. If files changed:
         - Commit per [commit-guidelines.md](commit-guidelines.md), referencing the CI failure
         - `git push`
         - Update `head_sha` and `last_push_time`, increment `fix_attempts[check_name]`
         - Add run ID to `handled_runs`
         - Log to `actions_log`: what failed, what was fixed

         If the subagent reports it could not fix the issue, log it and continue.

   f. **Check exit conditions**:
      - **All green**: every check passes, none pending, and `get-pr-comments.sh` returns zero unresolved threads -> report success, exit loop
      - **PR closed/merged**: `gh pr view --json state` shows MERGED or CLOSED -> report, exit loop
      - **Timeout**: max iterations reached -> report current status, exit loop

6. **Summary**: Report all actions from `actions_log`:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - CI jobs rerun (count, reasons: transient/flake)
   - Current CI status
   - Total commits pushed
