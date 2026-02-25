# Watch Operation

Monitor the current PR for CI failures and new review comments. Triage failures, fix issues, commit, and push autonomously.

## Instructions

1. **Verify PR**:
   ```bash
   gh pr view --json number,url,headRefOid,state
   ```
   If no PR exists, inform the user and stop.

2. **Detect CI system** per [git-patterns.md](git-patterns.md) "CI System Detection":
   ```bash
   repo_root=$(git rev-parse --show-toplevel)
   ```
   Check for `.github/workflows/` and `.buildkite/` to determine `ci_system` (`github-actions`, `buildkite`, or `unknown`). The `poll-pr-status` script detects this automatically and includes it in the `ci.ciSystem` field -- no separate detection needed at this step. Note that `ci` being null means no check runs were reported at all (rare), not that CI is unsupported.

3. **Snapshot initial state** using `poll-pr-status` (path in [git-patterns.md](git-patterns.md)):
   ```bash
   poll-pr-status
   ```
   From the returned JSON, initialize tracking state:
   - `handled_checks`: if `ci` is not null, pre-seed with names from `ci.pendingChecks` (avoids re-triaging in-progress checks). If `ci` is null (no check runs reported), initialize as empty.
   - `handled_threads`: empty set -- pre-existing unreplied threads are actionable and will surface on the first poll iteration
   - `fix_attempts`: empty map of check name to attempt count
   - `head_sha`: current HEAD (`git rev-parse HEAD`)
   - `last_push_time`: current timestamp (`date -u +%Y-%m-%dT%H:%M:%SZ`)
   - `actions_log`: empty list for the final summary

4. **Report initial status**: CI actionable status, pass/fail/pending counts, count of pre-existing unresolved threads, and that monitoring has started with 30s poll interval.

5. **Monitoring loop** (up to 90 iterations, ~45 minutes):

   a. **Sleep**: `sleep 30`
      If any API call in the previous iteration returned HTTP 429, double the sleep interval (up to 120s). Reset to 30s once a non-429 response is received.

   b. **Poll**: Run `poll-pr-status` (path in [git-patterns.md](git-patterns.md)) with current state:
      ```bash
      poll-pr-status --last-push-time <last_push_time> --handled-threads <id1,id2,...>
      ```
      Omit `--handled-threads` if the set is empty. The script filters stale failures (checks that started before `last_push_time`) and returns only threads not in the handled set.

   c. **Report poll result**: Log one line: `actionable=<value> new_failures=<N> new_threads=<N> pending=[<name>, ...]`. Do not dump raw JSON.

   d. **Handle new review threads** (if `threads.new > 0`): Follow the "Handle New Review Threads" protocol in [watch-protocol.md](watch-protocol.md). After handling, add each thread's last comment `id` to `handled_threads`.

   e. **Handle CI failures** (if any failure names not in `handled_checks`): Follow the "Handle CI Failures" protocol in [watch-protocol.md](watch-protocol.md). After handling, add the check name to `handled_checks`.

   f. **Check exit conditions** using the `exit` field from the poll response:
      - `exit == "all_green"`: all actionable checks pass, no new threads -> report success, exit loop
      - `exit == "pr_merged"` or `exit == "pr_closed"`: report, exit loop
      - `exit == null`: continue loop
      - **Timeout**: max iterations reached -> report current status, exit loop

6. **Summary**: Report all actions from `actions_log`:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - Current CI status
   - Total commits pushed
