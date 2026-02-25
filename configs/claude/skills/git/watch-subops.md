# Watch Sub-Operations

Procedures for handling review threads and CI failures during the watch loop. Referenced by [watch.md](watch.md).

All fixes MUST be delegated to subagents -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path.

## Context Management

The state file (`./tmp/ci-watch-<pr-number>.md`) is your persistent memory across iterations. Follow these rules strictly:

- **Read state at the start of each iteration.** The file is the source of truth, not your recollection.
- **Write state at the end of each iteration.** Every mutation (new handled check, new handled thread, updated fix attempts, new action log entry) goes into the file.
- **Do NOT retain raw poll JSON or subagent results in your working memory after processing.** Once you have extracted the relevant data and written it to the state file, discard the details. The state file is your record.
- **Do NOT summarize or accumulate iteration history in conversation.** If you need to recall what happened, read the state file.

This discipline keeps the context window small across 90-iteration sessions.

## State File Format

```markdown
# CI Watch State: PR #<number>

## Metadata
- head_sha: <sha>
- last_push_time: <iso timestamp>
- iteration: <n>
- started_at: <iso timestamp>

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

## State File Initialization

Check if `./tmp/ci-watch-<pr-number>.md` already exists.

**If it exists (resume):** Read the file. Use its values as the starting state. Run a fresh `poll-pr-status` call (using the file's `last_push_time` and `handled_threads`) and update the "Latest Status" section so monitoring begins with current data. Log a resume action: `- [<timestamp>] Resumed watch from iteration <n>`. Report to the user that a previous session is being resumed.

**If it does not exist (fresh start):** Run the initial `poll-pr-status` call, then create the file:

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

## Handle New Review Threads

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

## Handle CI Failures

For each failed check whose NAME is not in the "Handled Checks" section of the state file:

Note: when a new commit is pushed, checks from the previous commit may appear as CANCELLED. The `poll-pr-status` script filters these via `--last-push-time`, so CANCELLED checks from old commits will not appear as new failures. No special handling needed.

### Guard against infinite loops

If the "Fix Attempts" section in the state file shows `<check_name>: 2` or higher, log that repeated fixes have not resolved this check, skip it, and continue. Report to the user: "`<check name>` has failed after 2 fix attempts. Manual intervention may be needed. Monitoring continues for other checks."

### GitHub Actions (`ci_system == "github-actions"`)

#### Get the run database ID

Run the `get-failed-runs` script (path in [git-patterns.md](git-patterns.md)) with `--head-sha <current HEAD>` and `--check "<failed check name>"` to retrieve the run database ID.

```
get-failed-runs --head-sha <sha> --check "<check name>"
```

Returns a JSON array of `{ runId, workflowName, headSha, createdAt }`. If the array is empty, the check may be from a superseded commit -- log it, add the check name to the "Handled Checks" section in the state file, and skip.

#### Triage via ci-triager agent

Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `ci-triager` agent with:
- run_id: the failed run's database ID from the previous step
- workflow_name: the workflow name
- branch: current branch
- base_branch: detected base branch
- repo: owner/repo string

Based on the agent's classification:
- **transient** or **flake**: add check NAME to the "Handled Checks" section in the state file, append to the actions log: `- [<timestamp>] Classified <check name> as <classification>`. Continue.
- **real**: proceed to the "Fix a real failure" section below.

### Buildkite (`ci_system == "buildkite"`)

`gh run list` and `ci-triager` do not work for Buildkite. Instead:

1. Note the failed check name from the poll response.
2. Skip automated triage -- treat all Buildkite failures as real and proceed to the "Fix a real failure" section below.

### Unknown CI (`ci_system == "unknown"`)

Same as Buildkite: skip automated triage, treat failures as real, proceed to fix.

### Fix a real failure

Delegate to a Task subagent (subagent_type: general-purpose, model: sonnet) with:
- Failed check name (and trimmed logs + root cause from ci-triager, if available from the GitHub Actions path)
- Instruction: identify root cause, read relevant source files, fix the issue, run the appropriate lint-fix and test commands per [git-patterns.md](git-patterns.md) "Local Fix Commands", run local verification if possible

After the subagent returns, increment the count for `<check_name>` in the "Fix Attempts" section of the state file (this counts every attempt, regardless of outcome, so the infinite-loop guard triggers after 2 total tries). Then add check NAME to the "Handled Checks" section.

Check `git status --short`. If files changed:
- Spawn the `committer` agent with prompt: "Commit these changes. They fix a CI failure in <check name>: <brief failure summary>."
- `git push`
- Update `head_sha` and `last_push_time` in the state file
- Append to the "Actions Log" section: `- [<timestamp>] Fixed CI failure <check name>: <brief summary>`

If the subagent reports it could not fix the issue, append to the actions log: `- [<timestamp>] Could not fix <check name>: <reason>`. Continue.
