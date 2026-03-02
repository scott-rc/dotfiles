# Watch State Reference

State file format and monitoring loop protocol for the watch loop. Referenced by watch.md.

These rules apply within the watch loop context only. The watch loop always delegates to subagents to preserve long-running context.

## Delegation Pattern

The watch loop uses a two-level delegation pattern: the orchestrator reads state, triages, and dispatches; subagents debug and fix. Each subagent prompt includes the repository root path. Fixes MUST be delegated to subagents -- attempting fixes inline in the loop exhausts the context window and causes the loop to lose monitoring state.

State is persisted to the state file at `./tmp/ci-watch-<pr-number>.md` after each iteration. Raw poll data, subagent results, and intermediate JSON are discarded once relevant data is extracted and written to the state file.

## State File Format

```markdown
# CI Watch State: PR #<number>

## Metadata
- head_sha: <sha>
- last_push_time: <iso timestamp>
- iteration: <n>
- started_at: <iso timestamp>
- sleep_interval: <seconds> — AIMD poll interval (30–300); see Loop Protocol below

## Handled Checks
- <check name 1>
- <check name 2>

## Handled Threads
- <thread id 1>
- <thread id 2>

## Fix Attempts
- <check name>: <count>

## Actions Log
- [<timestamp>] <action description>
- [<timestamp>] <action description>

## Latest Status
<one-line status from most recent poll>
```

Each section uses simple markdown list items for easy reading and writing. Empty sections should contain no list items (just the heading).

## Loop Protocol

### AIMD Parameters

- `min_interval`: 30s
- `max_interval`: 300s
- `initial_interval`: 60s
- `additive_increase`: 30s -- added each idle iteration
- `multiplicative_decrease`: 0.5 -- multiplied when an event occurs

### Per-Iteration Steps

Each iteration follows this sequence:

1. Read the state file to load `sleep_interval`, `handled_checks`, `handled_threads`, `fix_attempts`, `head_sha`, `last_push_time`, `started_at`, and `iteration` count.
2. Run `sleep <sleep_interval>`.
3. Run `poll-pr-status` with current `--last-push-time`, `--handled-threads`, and `--handled-checks` values (extracted from bullet lists, joined with commas). Omit `--handled-threads` if empty; always pass `--handled-checks` (even empty -- the script handles it). The `--handled-checks` argument causes the script to use name-based filtering instead of timestamp filtering, which is critical for Buildkite where `startedAt` reflects the original job start.
4. **Detect new push**: compare the `headSha` field from the poll response (camelCase in JSON output) against the stored `head_sha` (snake_case in the state file). If they differ, a new push occurred outside watch -- clear `handled_checks` entirely (set to empty list) so all checks are re-evaluated against the new commit, update `head_sha` to the new value, and log: `- [<timestamp>] New push detected (<new sha>), cleared handled_checks`.
5. Report one line to the user AND update the "Latest Status" section: `actionable=<value> new_failures=<N> new_threads=<N> pending=[<name>, ...]`.
6. Handle new review threads (if `threads.new > 0`) -- see "Handling Review Threads" below.
7. Handle CI failures (if any failure names not in `handled_checks`) -- see "Handling CI Failures" below.
8. Check exit conditions using the `exit` field from the poll response:
   - `exit == "all_green"`: all actionable checks pass, no new threads -- report success, exit loop
   - `exit == "pr_merged"` or `exit == "pr_closed"`: report, exit loop
   - `exit == null`: continue loop
   - Timeout (max iterations reached): report current status, exit loop
9. Write state (MUST happen every iteration): increment `iteration`, update "Latest Status", compute new `sleep_interval`, write ALL state back to the file. No exceptions -- this enables the context-discard rule (raw poll data can be discarded once state is written).

### Handling Review Threads

When `threads.new > 0`, classify each new thread per references/bulk-threads.md:

- **Bot threads**: handle autonomously -- dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch", passing thread details (file path, line number, full comment bodies, last comment `comment_id`). After it returns, add the thread's last comment `id` to "Handled Threads" regardless of outcome. If files changed, spawn `committer` ("Commit these changes. They address PR review feedback: <brief summary>."), `git push`, and update `head_sha` and `last_push_time`. Then spawn one `github-writer` subagent (type: `review-reply`) for the thread using that thread's `comment_id`; reply text MUST follow references/github-text.md. Append to "Actions Log": `- [<timestamp>] Fixed review threads: <brief summary>, files: <list>`. If the subagent could not fix a thread, log it and continue.
- **Human reviewer threads**: Skip entirely. MUST NOT add to "Handled Threads", MUST NOT dispatch any subagent. Leave them for the standalone Fix Review operation.

### Handling CI Failures

When any failure names are not in `handled_checks`:

**Guard against infinite loops**: if "Fix Attempts" shows `<check_name>: 2` or higher, skip it and report to the user that manual intervention may be needed. Monitoring continues for other checks.

**GitHub Actions** (`ci_system == "github-actions"`): Run `get-failed-runs` (path in references/git-patterns.md) with `--head-sha <current HEAD>` and `--check "<failed check name>"`. If the result array is empty, verify whether the failing check's commit SHA matches the current HEAD. If the SHA does not match, the check is from a superseded commit -- log it, add to "Handled Checks", and skip. If the SHA matches or cannot be determined, the run may still be initializing -- MUST NOT add to handled_checks; skip this iteration and let the next poll pick it up. Detect base branch per references/git-patterns.md. Spawn `ci-triager` with: run_id, workflow_name, branch, base_branch, repo. If transient or flake: add check NAME to "Handled Checks", log the classification, continue. If real: proceed to fix.

**Buildkite** (`ci_system == "buildkite"`): Fetch logs per references/buildkite-handling.md. Proceed directly to fix.

**Unknown CI** (`ci_system == "unknown"`): Same as Buildkite.

**Fixing a real failure**: Dispatch a fix subagent per references/git-patterns.md "Fix Subagent Dispatch", passing the failed check name and trimmed logs/root cause from ci-triager (if available). After the subagent returns, increment `<check_name>` in "Fix Attempts" and add check NAME to "Handled Checks". If files changed: spawn `committer` ("Commit these changes. They fix a CI failure in <check name>: <brief failure summary>."), `git push`, update `head_sha` and `last_push_time`, append to "Actions Log". If the subagent could not fix the issue, log it and continue.

### Sleep Interval Computation

Compute new `sleep_interval` at end of each iteration:

If any API call returned HTTP 429: set `sleep_interval = max_interval` (300s).

Otherwise, determine whether the iteration was an event or idle:
- Event: new failures handled, OR new review threads handled, OR a push was made
- Idle: none of the above

Apply using the named parameters above:
- Event: `sleep_interval = max(sleep_interval * multiplicative_decrease, min_interval)`
- Idle: `sleep_interval = min(sleep_interval + additive_increase, max_interval)`
