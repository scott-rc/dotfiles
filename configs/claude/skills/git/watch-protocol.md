# Watch Protocol Reference

Detailed procedures for handling review threads and CI failures during the watch loop. Referenced by [watch.md](watch.md).

All fixes MUST be delegated to subagents -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path.

## Handle New Review Threads

Delegate to a Task subagent (type: general-purpose, model: sonnet) with:
- All new thread details: file path, line number, full comment bodies, last comment `id` per thread
- Instruction: read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify, then run the appropriate lint-fix command per [git-patterns.md](git-patterns.md) "Local Fix Commands"

After the subagent returns, collect the last comment `id` from each dispatched thread and add ALL of them to `handled_threads` (formatted as comma-separated string for the next poll). Add regardless of outcome -- prevents re-dispatching on next poll.

Check `git status --short`. If files changed:
- Spawn the `committer` agent with prompt: "Commit these changes. They address PR review feedback: <brief summary of threads fixed>."
- `git push`
- Update `head_sha` and `last_push_time`
- Reply to each fixed thread by spawning the `github-writer` agent with type `review-reply`, the brief message referencing the fix commit SHA as body, and target `owner`, `repo`, `comment_id` (using the `id` field from the last comment in each thread -- this is the REST database ID, suitable for the REST API reply endpoint).
- Log to `actions_log`: threads fixed, files touched

If the subagent reports it could not fix a thread, log it and continue -- do not block monitoring.

## Handle CI Failures

For each failed check whose NAME is not in `handled_checks`:

Note: when a new commit is pushed, checks from the previous commit may appear as CANCELLED. The `poll-pr-status` script filters these via `--last-push-time`, so CANCELLED checks from old commits will not appear as new failures. No special handling needed.

### Guard against infinite loops

If `fix_attempts[check_name] >= 2`, log that repeated fixes have not resolved this check, skip it, and continue. Report to the user that manual intervention may be needed.

### GitHub Actions (`ci_system == "github-actions"`)

#### Get the run database ID

Run the `get-failed-runs` script (path in [git-patterns.md](git-patterns.md)) with `--head-sha <current HEAD>` and `--check "<failed check name>"` to retrieve the run database ID.

```
get-failed-runs --head-sha <sha> --check "<check name>"
```

Returns a JSON array of `{ runId, workflowName, headSha, createdAt }`. If the array is empty, the check may be from a superseded commit -- log it, add the check name to `handled_checks`, and skip.

#### Triage via ci-triager agent

Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `ci-triager` agent with:
- run_id: the failed run's database ID from the previous step
- workflow_name: the workflow name
- branch: current branch
- base_branch: detected base branch
- repo: owner/repo string

Based on the agent's classification:
- **transient** or **flake**: add check NAME to `handled_checks`, log the classification and indicator, continue.
- **real**: proceed to the "Fix a real failure" section below.

### Buildkite (`ci_system == "buildkite"`)

`gh run list` and `ci-triager` do not work for Buildkite. Instead:

1. Note the failed check name from the poll response.
2. Skip automated triage -- treat all Buildkite failures as real and proceed to the "Fix a real failure" section below.

### Unknown CI (`ci_system == "unknown"`)

Same as Buildkite: skip automated triage, treat failures as real, proceed to fix.

### Fix a real failure

Delegate to a Task subagent (type: general-purpose, model: sonnet) with:
- Failed check name (and trimmed logs + root cause from ci-triager, if available from the GitHub Actions path)
- Instruction: identify root cause, read relevant source files, fix the issue, run the appropriate lint-fix and test commands per [git-patterns.md](git-patterns.md) "Local Fix Commands", run local verification if possible

After the subagent returns, check `git status --short`. If files changed:
- Spawn the `committer` agent with prompt: "Commit these changes. They fix a CI failure in <check name>: <brief failure summary>."
- `git push`
- Update `head_sha` and `last_push_time`, increment `fix_attempts[check_name]`
- Add check NAME to `handled_checks`
- Log to `actions_log`: what failed, what was fixed

If the subagent reports it could not fix the issue, log it and continue.
