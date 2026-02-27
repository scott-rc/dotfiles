# Watch

Monitor the current PR for CI failures and new review comments. Triage failures, fix issues, commit, and push autonomously.

All tracking state lives in a temporary state file (`./tmp/ci-watch-<pr-number>.md`), not in the conversation. Each iteration reads state from the file, processes the poll, and writes updated state back. This keeps the context window small across long monitoring sessions. Read the state file at iteration start, write it back after processing. Do NOT retain raw JSON, subagent results, or poll details in working memory after extracting relevant data into the state file -- discard them. If you need to recall what happened, read the state file.

All fixes MUST be delegated to subagents -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path.

When the loop encounters review threads, always delegate to a subagent rather than handling inline. The watch loop must preserve its long-running context; unlike the standalone review operation (which may handle simple cases inline), watch always delegates.

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

   **If it does not exist (fresh start):** Run the initial `poll-pr-status` call, then create the file with the fields defined in references/watch-subops.md. Populate `head_sha` from `git rev-parse HEAD`, set `last_push_time` and `started_at` to the current UTC timestamp, `iteration` to 0, `sleep_interval` to `initial_interval` (60), and `actions_log` with a single `Watch started` entry. For `handled_checks`: on `github-actions` CI, if `ci` is not null, pre-seed with names of currently-pending checks (from `ci.pendingChecks`) to avoid re-triaging in-progress checks -- current failures are NOT pre-seeded. On `buildkite` and `unknown` CI, start with empty `handled_checks` (pre-seeding the Buildkite umbrella check name while pending would mark it as handled if it later fails). If `ci` is null, leave empty.

   Ensure the `./tmp/` directory exists before writing (`mkdir -p ./tmp`). If this fails, inform the user and stop -- the state file cannot be created.

4. **Report initial status**: CI actionable status, pass/fail/pending counts, count of pre-existing unresolved threads, and that monitoring has started with adaptive AIMD polling (30s-300s, starting at 60s). If resuming, note the previous iteration count and any prior actions.

5. **Run the monitoring loop** per references/watch-subops.md (up to 30 iterations, ~60 minutes at average idle interval). Follow the per-iteration steps and sleep interval computation defined there.

   **Handle new review threads** (if `threads.new > 0`):
   Resolve the local fix commands from references/git-patterns.md "Local Fix Commands" section and pass them inline in the subagent prompt. Spawn a general-purpose subagent (model: sonnet) with: all new thread details (file path, line number, full comment bodies, last comment `comment_id` per thread), the repository root path, and instruction to read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify using the resolved lint and test commands, and consult the project's CLAUDE.md for project-specific commands.

   After the subagent returns, add the last comment `id` from each dispatched thread to the "Handled Threads" section in the state file (regardless of outcome -- prevents re-dispatching). Check `git status --short`. If files changed: spawn the `committer` agent ("Commit these changes. They address PR review feedback: <brief summary>."), `git push`, update `head_sha` and `last_push_time`, spawn `github-writer` (type: `review-reply`) for each fixed thread using the `comment_id` from the last comment in that thread. If `github-writer` returns an error, log it to the actions log and continue. Append to the "Actions Log": `- [<timestamp>] Fixed review threads: <brief summary>, files: <list>`. If the subagent could not fix a thread, log it and continue.

   **Handle CI failures** (if any failure names not in `handled_checks`):
   Note: CANCELLED checks from old commits are filtered by `poll-pr-status` via `--last-push-time` -- no special handling needed.

   Guard against infinite loops: if "Fix Attempts" shows `<check_name>: 2` or higher, skip it and report to the user that manual intervention may be needed. Monitoring continues for other checks.

   For GitHub Actions (`ci_system == "github-actions"`): run `get-failed-runs` (path in references/git-patterns.md) with `--head-sha <current HEAD>` and `--check "<failed check name>"`. If the result array is empty, the check is from a superseded commit -- log it, add to "Handled Checks", and skip. Detect base branch per references/git-patterns.md. Spawn `ci-triager` with: run_id, workflow_name, branch, base_branch, repo. If transient or flake: add check NAME to "Handled Checks", log the classification, continue. If real: proceed to fix.

   For Buildkite (`ci_system == "buildkite"`): see references/buildkite-handling.md for log fetching and umbrella check handling. Skip automated triage -- treat all failures as real and proceed to fix.

   For unknown CI (`ci_system == "unknown"`): same as Buildkite -- skip triage, treat failures as real, proceed to fix.

   To fix a real failure: resolve the local fix commands from references/git-patterns.md "Local Fix Commands" section and pass them inline in the subagent prompt. Spawn a general-purpose subagent (model: sonnet) with: failed check name, trimmed logs and root cause from ci-triager (if available), and instruction to identify root cause, read relevant source files, fix the issue, run the resolved lint and test commands, and consult the project's CLAUDE.md. After the subagent returns, increment `<check_name>` in "Fix Attempts" and add check NAME to "Handled Checks". Check `git status --short`. If files changed: spawn `committer` ("Commit these changes. They fix a CI failure in <check name>: <brief failure summary>."), `git push`, update `head_sha` and `last_push_time`, append to "Actions Log". If the subagent could not fix the issue, log it and continue.

6. **Summary**: Read the actions log from the state file and report:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - Current CI status
   - Total commits pushed
   - State file path (`./tmp/ci-watch-<pr-number>.md`) so the user can review the full log
