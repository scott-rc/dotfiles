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

   d. **Handle new review threads** (if any): Follow the "Handle New Review Threads" protocol in [watch-protocol.md](watch-protocol.md).

   e. **Handle CI failures** (if any new failures not in `handled_runs`): Follow the "Handle CI Failures" protocol in [watch-protocol.md](watch-protocol.md).

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
