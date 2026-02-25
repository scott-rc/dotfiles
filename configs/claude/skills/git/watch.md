# Watch Operation

Monitor the current PR for CI failures and new review comments. Triage failures, fix issues, commit, and push autonomously.

All tracking state lives in a temporary state file (`./tmp/ci-watch-<pr-number>.md`), not in the conversation. Each iteration reads state from the file, processes the poll, and writes updated state back. This keeps the context window small across long monitoring sessions.

## Instructions

1. **Verify PR**:
   ```bash
   gh pr view --json number,url,headRefOid,state
   ```
   If no PR exists, inform the user and stop. Save the PR number for the state file path.

2. **CI system**: `poll-pr-status` detects the CI system automatically and includes it in the `ci.ciSystem` field (`github-actions`, `buildkite`, or `unknown`). No separate detection step is needed. Note that `ci` being null means no check runs were reported at all (rare), not that CI is unsupported.

3. **Initialize or resume state file**: Follow the "State File Initialization" procedure in [watch-subops.md](watch-subops.md). This creates `./tmp/ci-watch-<pr-number>.md` with initial values from the first `poll-pr-status` call, or resumes from an existing file if one is found.

4. **Report initial status**: CI actionable status, pass/fail/pending counts, count of pre-existing unresolved threads, and that monitoring has started with 30s poll interval. If resuming, note the previous iteration count and any prior actions.

5. **Monitoring loop** (up to 90 iterations, ~45 minutes):

   a. **Sleep**: `sleep 30`
      If any API call in the previous iteration returned HTTP 429, double the sleep interval (up to 120s). Reset to 30s once a non-429 response is received.

   b. **Read state**: Read the state file to load current `handled_checks`, `handled_threads`, `fix_attempts`, `head_sha`, `last_push_time`, `started_at`, and `iteration` count.

   c. **Poll**: Run `poll-pr-status` (path in [git-patterns.md](git-patterns.md)) with current state:
      ```bash
      poll-pr-status --last-push-time <last_push_time> --handled-threads <id1,id2,...>
      ```
      The `handled_threads` in the state file are markdown bullet items (`- <id>`). Extract the IDs and join them with commas for the `--handled-threads` argument. Omit `--handled-threads` if the set is empty. The script filters stale failures (checks that started before `last_push_time`) and returns only threads not in the handled set.

   d. **Report poll result**: Report one line to the user AND update the "Latest Status" section in the state file: `actionable=<value> new_failures=<N> new_threads=<N> pending=[<name>, ...]`. Do not dump raw JSON.

   e. **Handle new review threads** (if `threads.new > 0`): Follow the "Handle New Review Threads" protocol in [watch-subops.md](watch-subops.md). After handling, add each thread's last comment `id` to `handled_threads` in the state file.

   f. **Handle CI failures** (if any failure names not in `handled_checks`): Follow the "Handle CI Failures" protocol in [watch-subops.md](watch-subops.md). After handling, add the check name to `handled_checks` in the state file.

   g. **Check exit conditions** using the `exit` field from the poll response:
      - `exit == "all_green"`: all actionable checks pass, no new threads -> report success, exit loop
      - `exit == "pr_merged"` or `exit == "pr_closed"`: report, exit loop
      - `exit == null`: continue loop
      - **Timeout**: max iterations reached -> report current status, exit loop

   h. **Write state**: Increment `iteration`, write ALL updated state back to the file.

6. **Summary**: Read the actions log from the state file and report:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - Current CI status
   - Total commits pushed
   - State file path (`./tmp/ci-watch-<pr-number>.md`) so the user can review the full log
