# Watch State Reference

State file format and monitoring loop protocol for the watch loop. Referenced by watch.md.

These rules apply within the watch loop context only. The standalone Fix Review operation handles simple cases inline; the watch loop always delegates to preserve long-running context.

## Delegation Pattern

The watch loop uses a two-level delegation pattern: the orchestrator reads state, triages, and dispatches; subagents debug and fix. Each subagent prompt includes the repository root path. All fixes go to subagents -- attempting fixes inline in the loop exhausts the context window and causes the loop to lose monitoring state.

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
4. Report one line to the user AND update the "Latest Status" section: `actionable=<value> new_failures=<N> new_threads=<N> pending=[<name>, ...]`.
5. Handle new review threads (if `threads.new > 0`) -- see watch.md for delegation details.
6. Handle CI failures (if any failure names not in `handled_checks`) -- see watch.md for delegation details.
7. Check exit conditions using the `exit` field from the poll response:
   - `exit == "all_green"`: all actionable checks pass, no new threads -- report success, exit loop
   - `exit == "pr_merged"` or `exit == "pr_closed"`: report, exit loop
   - `exit == null`: continue loop
   - Timeout (max iterations reached): report current status, exit loop
8. Write state (MUST happen every iteration): increment `iteration`, update "Latest Status", compute new `sleep_interval`, write ALL state back to the file. No exceptions -- this enables the context-discard rule (raw poll data can be discarded once state is written).

### Sleep Interval Computation

Compute new `sleep_interval` at end of each iteration:

If any API call returned HTTP 429: set `sleep_interval = max_interval` (300s).

Otherwise, determine whether the iteration was an event or idle:
- Event: new failures handled, OR new review threads handled, OR a push was made
- Idle: none of the above

Apply:
- Event: `sleep_interval = max(sleep_interval * multiplicative_decrease, min_interval)`
- Idle: `sleep_interval = min(sleep_interval + additive_increase, max_interval)`
