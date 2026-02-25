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

3. **Initialize or resume state file**:
   Check if `./tmp/ci-watch-<pr-number>.md` already exists.

   **If it exists (resume):** Read the file. Use its values as the starting state. Run a fresh `poll-pr-status` call (using the file's `last_push_time` and `handled_threads`) and update the "Latest Status" section so monitoring begins with current data. Log a resume action: `- [<timestamp>] Resumed watch from iteration <n>`. Report to the user that a previous session is being resumed.

   **If it does not exist (fresh start):** Run the initial `poll-pr-status` call, then create the file using the format in [watch-subops.md](watch-subops.md):

   1. `head_sha`: `git rev-parse HEAD`
   2. `last_push_time`: `date -u +%Y-%m-%dT%H:%M:%SZ`
   3. `iteration`: 0
   4. `started_at`: same as `last_push_time`
   5. `handled_checks`: if `ci` is not null, pre-seed with names of currently-pending checks only (from `ci.pendingChecks`) -- this avoids re-triaging in-progress checks. Current failures are NOT pre-seeded; they will be handled in the first monitoring iteration. If `ci` is null, leave empty.
   6. `handled_threads`: empty
   7. `fix_attempts`: empty
   8. `actions_log`: one entry: `- [<timestamp>] Watch started`
   9. `latest_status`: initial CI summary line

   Ensure the `./tmp/` directory exists before writing (`mkdir -p ./tmp`). If this fails, inform the user and stop -- the state file cannot be created.

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

   e. **Handle new review threads** (if `threads.new > 0`):
      Delegate to a Task subagent (subagent_type: general-purpose, model: sonnet) with:
      - All new thread details: file path, line number, full comment bodies, last comment `id` per thread
      - Instruction: read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify, then run the appropriate lint-fix command per [git-patterns.md](git-patterns.md) "Local Fix Commands"

      After the subagent returns, collect the last comment `id` from each dispatched thread and add ALL of them to the "Handled Threads" section in the state file. Add regardless of outcome -- prevents re-dispatching on next poll.

      Check `git status --short`. If files changed:
      - Spawn the `committer` agent with prompt: "Commit these changes. They address PR review feedback: <brief summary of threads fixed>."
      - `git push`
      - Update `head_sha` and `last_push_time` in the state file
      - Reply to each fixed thread by spawning the `github-writer` agent with type `review-reply`, the brief message referencing the fix commit SHA as body, and target `owner`, `repo`, `comment_id` (using the `id` field from the last comment in each thread -- this is the REST database ID, suitable for the REST API reply endpoint).
      - Append to the "Actions Log" section in the state file: `- [<timestamp>] Fixed review threads: <brief summary>, files: <list>`

      If the subagent reports it could not fix a thread, append to the actions log: `- [<timestamp>] Could not fix thread <id>: <reason>`. Continue -- do not block monitoring.

   f. **Handle CI failures** (if any failure names not in `handled_checks`):
      For each failed check whose NAME is not in the "Handled Checks" section of the state file:

      Note: when a new commit is pushed, checks from the previous commit may appear as CANCELLED. The `poll-pr-status` script filters these via `--last-push-time`, so CANCELLED checks from old commits will not appear as new failures. No special handling needed.

      **Guard against infinite loops:** If the "Fix Attempts" section in the state file shows `<check_name>: 2` or higher, log that repeated fixes have not resolved this check, skip it, and continue. Report to the user: "`<check name>` has failed after 2 fix attempts. Manual intervention may be needed. Monitoring continues for other checks."

      **GitHub Actions (`ci_system == "github-actions"`):**

      Run the `get-failed-runs` script (path in [git-patterns.md](git-patterns.md)) with `--head-sha <current HEAD>` and `--check "<failed check name>"` to retrieve the run database ID. Returns a JSON array of `{ runId, workflowName, headSha, createdAt }`. If the array is empty, the check may be from a superseded commit -- log it, add the check name to the "Handled Checks" section in the state file, and skip.

      Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `ci-triager` agent with: run_id, workflow_name, branch, base_branch, repo. Based on the agent's classification:
      - **transient** or **flake**: add check NAME to the "Handled Checks" section in the state file, append to the actions log: `- [<timestamp>] Classified <check name> as <classification>`. Continue.
      - **real**: proceed to the fix step below.

      **Buildkite (`ci_system == "buildkite"`):** `gh run list` and `ci-triager` do not work for Buildkite. Note the failed check name from the poll response. Skip automated triage -- treat all Buildkite failures as real and proceed to fix.

      **Unknown CI (`ci_system == "unknown"`):** Same as Buildkite: skip automated triage, treat failures as real, proceed to fix.

      **Fix a real failure:** Delegate to a Task subagent (subagent_type: general-purpose, model: sonnet) with:
      - Failed check name (and trimmed logs + root cause from ci-triager, if available from the GitHub Actions path)
      - Instruction: identify root cause, read relevant source files, fix the issue, run the appropriate lint-fix and test commands per [git-patterns.md](git-patterns.md) "Local Fix Commands", run local verification if possible

      After the subagent returns, increment the count for `<check_name>` in the "Fix Attempts" section of the state file (this counts every attempt, regardless of outcome, so the infinite-loop guard triggers after 2 total tries). Then add check NAME to the "Handled Checks" section.

      Check `git status --short`. If files changed:
      - Spawn the `committer` agent with prompt: "Commit these changes. They fix a CI failure in <check name>: <brief failure summary>."
      - `git push`
      - Update `head_sha` and `last_push_time` in the state file
      - Append to the "Actions Log" section: `- [<timestamp>] Fixed CI failure <check name>: <brief summary>`

      If the subagent reports it could not fix the issue, append to the actions log: `- [<timestamp>] Could not fix <check name>: <reason>`. Continue.

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
