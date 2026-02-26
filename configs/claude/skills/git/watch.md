# Watch Operation

Monitor the current PR for CI failures and new review comments. Triage failures, fix issues, commit, and push autonomously.

All tracking state lives in a temporary state file (`./tmp/ci-watch-<pr-number>.md`), not in the conversation. Each iteration reads state from the file, processes the poll, and writes updated state back. This keeps the context window small across long monitoring sessions.

## Context Management

The state file is your persistent memory across iterations. Read it at iteration start, write it back after processing. Do NOT retain raw JSON, subagent results, or poll details in your working memory after extracting relevant data into the state file -- discard them. If you need to recall what happened, read the state file. This keeps context small across 90-iteration sessions.

All fixes MUST be delegated to subagents -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path.

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

   **If it does not exist (fresh start):** Run the initial `poll-pr-status` call, then create the file with the fields defined in [watch-subops.md](watch-subops.md). Populate `head_sha` from `git rev-parse HEAD`, set `last_push_time` and `started_at` to the current UTC timestamp, `iteration` to 0, `sleep_interval` to `initial_interval` (30), and `actions_log` with a single `Watch started` entry. For `handled_checks`: on `github-actions` CI, if `ci` is not null, pre-seed with names of currently-pending checks (from `ci.pendingChecks`) to avoid re-triaging in-progress checks -- current failures are NOT pre-seeded. On `buildkite` and `unknown` CI, start with empty `handled_checks` (pre-seeding the Buildkite umbrella check name while pending would mark it as handled if it later fails). If `ci` is null, leave empty.

   Ensure the `./tmp/` directory exists before writing (`mkdir -p ./tmp`). If this fails, inform the user and stop -- the state file cannot be created.

4. **Report initial status**: CI actionable status, pass/fail/pending counts, count of pre-existing unresolved threads, and that monitoring has started with adaptive AIMD polling (10s–120s, starting at 30s). If resuming, note the previous iteration count and any prior actions.

5. **Monitoring loop** (up to 90 iterations, ~45 minutes):

   AIMD parameters:
   - `min_interval`: 10s
   - `max_interval`: 120s
   - `initial_interval`: 30s
   - `additive_increase`: 5s — added each idle iteration
   - `multiplicative_decrease`: 0.5 — multiplied when an event occurs

   a. **Read state**: Read the state file to load `sleep_interval`, `handled_checks`, `handled_threads`, `fix_attempts`, `head_sha`, `last_push_time`, `started_at`, and `iteration` count.

   b. **Sleep**: Run `sleep <sleep_interval>`.

   c. **Poll**: Run `poll-pr-status` (path in [git-patterns.md](git-patterns.md)) with current state:
      ```bash
      poll-pr-status --last-push-time <last_push_time> --handled-threads <id1,id2,...> --handled-checks <name1,name2,...>
      ```
      The `handled_threads` and `handled_checks` in the state file are markdown bullet items (`- <value>`). Extract the values and join them with commas for the arguments. Omit `--handled-threads` if the set is empty. Always pass `--handled-checks` with the current handled checks set (even if empty -- the script handles empty strings gracefully). When `--handled-checks` is provided, the script uses it instead of timestamp filtering to determine new failures, which is critical for Buildkite where `startedAt` reflects the original job start, not retries.

   d. **Report poll result**: Report one line to the user AND update the "Latest Status" section in the state file: `actionable=<value> new_failures=<N> new_threads=<N> pending=[<name>, ...]`. Do not dump raw JSON.

   e. **Handle new review threads** (if `threads.new > 0`):
      Spawn a general-purpose subagent (model: sonnet) with:
      - All new thread details: file path, line number, full comment bodies, last comment `id` per thread
      - The repository root path
      - Instruction: read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify, then run the appropriate lint-fix command per [git-patterns.md](git-patterns.md) "Local Fix Commands"

      After the subagent returns, collect the last comment `id` from each dispatched thread and add ALL of them to the "Handled Threads" section in the state file. Add regardless of outcome -- prevents re-dispatching on next poll.

      Check `git status --short`. If files changed:
      - Spawn the `committer` agent with prompt: "Commit these changes. They address PR review feedback: <brief summary of threads fixed>."
      - `git push`
      - Update `head_sha` and `last_push_time` in the state file
      - Reply to each fixed thread by spawning the `github-writer` agent with type `review-reply`, the brief message referencing the fix commit SHA as body, and target `owner`, `repo`, `comment_id` (using the `id` field from the last comment in each thread -- this is the REST database ID, suitable for the REST API reply endpoint). If `github-writer` returns an error, log the failure to the actions log in the state file (`- [<timestamp>] Failed to reply to thread <id>: <error>`) and continue. Do NOT mark the thread as un-handled -- it was already added to "Handled Threads" above and should not be re-dispatched.
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

      To get failure logs from Buildkite, use the project-local `buildkite` CLI script (a Node.js script that queries the Buildkite API for failed jobs and their logs; locate it under the project's `.ai/skills/ci/` directory if it exists -- see [git-patterns.md](git-patterns.md)):
      1. Get the build URL: `gh pr checks --json name,state,detailsUrl | jq -r '.[] | select(.state == "FAILURE") | .detailsUrl'`
      2. Parse the org, pipeline, and build number from the URL (format: `https://buildkite.com/<org>/<pipeline>/builds/<number>...`)
      3. List failed jobs: `direnv exec . <buildkite-script-path> failed <org> <pipeline> <build-number>`
      4. Get logs for all failed jobs: `direnv exec . <buildkite-script-path> failed-logs <org> <pipeline> <build-number>`
      5. Pass the trimmed failure logs and root cause to the fix subagent, same as the GitHub Actions path

      **Important:** Buildkite runs sharded checks (e.g. ~80 parallel "node-api" jobs) that share the same check name in GitHub. A single failed shard gets masked by subsequent passing shards because GitHub reports the latest check result per name, not the worst. The `failed` count in the poll response correctly counts all failures, but `gh pr checks` may show the check as passing. Always use the `buildkite` script to inspect the actual build when the poll shows `failed > 0`, even if all named checks appear green.

      **Unknown CI (`ci_system == "unknown"`):** Same as Buildkite: skip automated triage, treat failures as real, proceed to fix.

      **Fix a real failure:** Spawn a general-purpose subagent (model: sonnet) with:
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

      **Buildkite umbrella check:** When `ciSystem == "buildkite"`, the umbrella parent check (e.g. `buildkite/gadget`) often stays in FAILURE state even after all child jobs pass on retry. If `exit` is null and the only failing check is the umbrella parent, and all other actionable checks are passing, treat it as effectively `all_green`. To detect this: check if `ci.failed == 1` and the single failure name matches `^buildkite/[^/]+$` (exactly two slash-separated components -- e.g. `buildkite/gadget` matches, but `buildkite/gadget/node-api` does not). Add the umbrella check to handled_checks so it does not block future iterations either.

   h. **Write state** (MUST happen every iteration): Increment `iteration`, update the "Latest Status" line, compute the new `sleep_interval`, and write ALL state back to the file. This MUST happen at the end of EVERY iteration, even if nothing changed -- no exceptions. At minimum, the iteration counter, Latest Status line, and sleep_interval must be updated. This ensures resume-after-crash loses at most one iteration of progress, and enables the context-discard rule (see Context Management above: once state is written, raw poll data can be discarded).

      **Compute new `sleep_interval`:**

      If any API call this iteration returned HTTP 429: set `sleep_interval = max_interval` (120s).

      Otherwise, determine whether this was an event iteration or idle iteration:
      - **Event**: new failures were handled, OR new review threads were handled, OR a push was made
      - **Idle**: none of the above occurred

      Apply the formula:
      - Event: `sleep_interval = max(sleep_interval * multiplicative_decrease, min_interval)`
      - Idle: `sleep_interval = min(sleep_interval + additive_increase, max_interval)`

6. **Summary**: Read the actions log from the state file and report:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - Current CI status
   - Total commits pushed
   - State file path (`./tmp/ci-watch-<pr-number>.md`) so the user can review the full log
