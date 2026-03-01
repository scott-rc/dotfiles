# Watch

Monitor the current PR for CI failures and new review comments. Triage failures, fix issues, commit, and push autonomously.

## Instructions

1. **Verify PR**:
   ```bash
   gh pr view --json number,url,headRefOid,state
   ```
   If no PR exists, inform the user and stop. Save the PR number for the state file path.

2. **CI system**: `poll-pr-status` detects the CI system automatically and includes it in the `ci.ciSystem` field (`github-actions`, `buildkite`, or `unknown`). No separate detection step is needed. Note that `ci` being null means no check runs were reported at all (rare), not that CI is unsupported.

3. **Initialize or resume state file** (state file format defined in references/watch-subops.md):
   Check if `./tmp/ci-watch-<pr-number>.md` already exists.

   **If it exists (resume):** Read the file. Use its values as the starting state. Run a fresh `poll-pr-status` call (using the file's `last_push_time` and `handled_threads`) and update the "Latest Status" section so monitoring begins with current data. Log a resume action: `- [<timestamp>] Resumed watch from iteration <n>`. Report to the user that a previous session is being resumed.

   **If it does not exist (fresh start):** Run the initial `poll-pr-status` call, then create the file with the fields defined in references/watch-subops.md. Populate `head_sha` from `git rev-parse HEAD`, set `last_push_time` and `started_at` to the current UTC timestamp, `iteration` to 0, `sleep_interval` to `initial_interval` (60), and `actions_log` with a single `Watch started` entry. Start `handled_checks` empty for all CI systems -- do not pre-seed pending checks, as a pending check that later transitions to failed must be handled normally and pre-seeding it into `handled_checks` would cause it to be silently skipped.

   MUST create `./tmp/` before writing (`mkdir -p ./tmp`). If this fails, inform the user and stop -- the state file cannot be created.

4. **Report initial status**: CI actionable status, pass/fail/pending counts, count of pre-existing unresolved threads, and that monitoring has started with adaptive AIMD polling (30s-300s, starting at 60s). If resuming, note the previous iteration count and any prior actions.

5. **Run the monitoring loop** (up to 30 iterations, ~60 minutes at average idle interval). Each iteration: sleep, poll `poll-pr-status`, handle new review threads and CI failures by dispatching subagents, write state, then check exit conditions. Follow the per-iteration steps, thread/CI handling, and sleep interval computation defined in references/watch-subops.md.

6. **Summary**: Read the actions log from the state file and report:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - Current CI status
   - Total commits pushed
   - State file path (`./tmp/ci-watch-<pr-number>.md`) so the user can review the full log
