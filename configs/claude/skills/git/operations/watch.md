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

   Ensure the `./tmp/` directory exists before writing (`mkdir -p ./tmp`). If this fails, inform the user and stop -- the state file cannot be created.

4. **Report initial status**: CI actionable status, pass/fail/pending counts, count of pre-existing unresolved threads, and that monitoring has started with adaptive AIMD polling (30s-300s, starting at 60s). If resuming, note the previous iteration count and any prior actions.

5. **Run the monitoring loop** (loop protocol and AIMD parameters in references/watch-subops.md; up to 30 iterations, ~60 minutes at average idle interval). Follow the per-iteration steps and sleep interval computation defined there.

   **Handle new review threads** (if `threads.new > 0`):
   Dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch", passing all new thread details (file path, line number, full comment bodies, last comment `comment_id` per thread). After it returns, add each thread's last comment `id` to "Handled Threads" regardless of outcome. If files changed, spawn `committer` ("Commit these changes. They address PR review feedback: <brief summary>."), `git push`, and update `head_sha` and `last_push_time`. Then spawn one `github-writer` subagent (type: `review-reply`) per thread using that thread's `comment_id`; reply text MUST follow references/github-text.md. Log any errors and continue. Append to "Actions Log": `- [<timestamp>] Fixed review threads: <brief summary>, files: <list>`. If the subagent could not fix a thread, log it and continue.

   **Handle CI failures** (if any failure names not in `handled_checks`):
   Note: CANCELLED checks from old commits are filtered by `poll-pr-status` via `--last-push-time` -- no special handling needed.

   Guard against infinite loops: if "Fix Attempts" shows `<check_name>: 2` or higher, skip it and report to the user that manual intervention may be needed. Monitoring continues for other checks.

   For GitHub Actions (`ci_system == "github-actions"`): run `get-failed-runs` (path in references/git-patterns.md) with `--head-sha <current HEAD>` and `--check "<failed check name>"`. If the result array is empty, verify whether the failing check's commit SHA matches the current HEAD. If the SHA does not match, the check is from a superseded commit -- log it, add to "Handled Checks", and skip. If the SHA matches or cannot be determined, the run may still be initializing -- do NOT add to handled_checks; skip this iteration and let the next poll pick it up. Detect base branch per references/git-patterns.md. Spawn `ci-triager` with: run_id, workflow_name, branch, base_branch, repo. If transient or flake: add check NAME to "Handled Checks", log the classification, continue. If real: proceed to fix.

   In-progress runs with failed jobs: `get-failed-runs` includes these, but full logs are unavailable until the run completes. ci-triager falls back to annotations (file paths and error descriptions) for these runs -- the fix subagent should expect annotation-level detail rather than full stack traces.

   For Buildkite (`ci_system == "buildkite"`): fetch logs via the buildkite reference path in references/buildkite-handling.md (do NOT use ci-triager). Skip automated triage -- treat all failures as real and proceed to fix using those logs.

   For unknown CI (`ci_system == "unknown"`): same as Buildkite -- skip triage, treat failures as real, proceed to fix.

   To fix a real failure: dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch", passing the failed check name and trimmed logs/root cause from ci-triager (if available). After the subagent returns, increment `<check_name>` in "Fix Attempts" and add check NAME to "Handled Checks". If files changed: spawn `committer` ("Commit these changes. They fix a CI failure in <check name>: <brief failure summary>."), `git push`, update `head_sha` and `last_push_time`, append to "Actions Log". If the subagent could not fix the issue, log it and continue.

6. **Summary**: Read the actions log from the state file and report:
   - Review threads addressed (count, files)
   - CI failures fixed (count, root causes)
   - Current CI status
   - Total commits pushed
   - State file path (`./tmp/ci-watch-<pr-number>.md`) so the user can review the full log
