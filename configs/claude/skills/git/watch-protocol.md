# Watch Protocol Reference

Detailed procedures for handling review threads and CI failures during the watch loop. Referenced by [watch.md](watch.md).

All fixes MUST be delegated to subagents -- reading source files and attempting fixes inline exhausts the context window and causes the loop to lose track of its monitoring state. The orchestrator triages and dispatches; subagents debug and fix. Each subagent prompt MUST include the repository root path.

## Handle New Review Threads

Delegate to a Task subagent (type: general-purpose, model: sonnet) with:
- All new thread details: file path, line number, full comment bodies, last comment `id` per thread
- Instruction: read each file, understand the reviewer's concern, apply the fix, run relevant tests to verify

After the subagent returns, add each thread's last comment `id` to `handled_threads` regardless of outcome (prevents re-dispatching on next poll).

Check `git status --short`. If files changed:
- Spawn the `committer` agent with prompt: "Commit these changes. They address PR review feedback: <brief summary of threads fixed>."
- `git push`
- Update `head_sha` and `last_push_time`
- Reply to each fixed thread by spawning the `github-writer` agent with type `review-reply`, the brief message referencing the fix commit SHA as body, and target `owner`, `repo`, `comment_id` (using the REST `databaseId` from `get-pr-comments.sh`, not the GraphQL node ID).
- Log to `actions_log`: threads fixed, files touched

If the subagent reports it could not fix a thread, log it and continue -- do not block monitoring.

## Handle CI Failures

For each failed check not in `handled_runs`:

### Guard against infinite loops

If `fix_attempts[check_name] >= 2`, log that repeated fixes have not resolved this check, skip it, and continue. Report to the user that manual intervention may be needed.

### Triage via ci-triager agent

Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `ci-triager` agent with:
- run_id: the failed run's database ID
- workflow_name: the workflow name
- branch: current branch
- base_branch: detected base branch
- repo: owner/repo string

Based on the agent's classification:
- **transient** or **flake**: add run ID to `handled_runs`, log the classification and indicator, continue.
- **real**: proceed to fix.

### Real failure -- fix it

Delegate to a Task subagent (type: general-purpose, model: sonnet) with:
- Trimmed logs and root cause from the ci-triager's report
- Workflow name
- Instruction: identify root cause, read relevant source files, fix the issue, run local verification if possible

After the subagent returns, check `git status --short`. If files changed:
- Spawn the `committer` agent with prompt: "Commit these changes. They fix a CI failure in <workflow>/<step>: <brief failure summary>."
- `git push`
- Update `head_sha` and `last_push_time`, increment `fix_attempts[check_name]`
- Add run ID to `handled_runs`
- Log to `actions_log`: what failed, what was fixed

If the subagent reports it could not fix the issue, log it and continue.
