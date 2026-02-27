# Buildkite CI Handling

How to fetch logs and handle failures from Buildkite CI in the watch loop.

## Overview

`gh run list`, `ci-triager`, and `get-failed-runs` do not work for Buildkite. Treat all Buildkite failures as real and proceed directly to the fix step. Skip automated triage.

Note: Buildkite runs sharded checks (e.g., ~80 parallel "node-api" jobs) that share the same check name in GitHub. A single failed shard gets masked by subsequent passing shards because GitHub reports the latest check result per name, not the worst. The `failed` count in the poll response correctly counts all failures, but `gh pr checks` may show the check as passing. Always use the `buildkite` script to inspect the actual build when the poll shows `failed > 0`, even if all named checks appear green.

## Locating the buildkite Script

A project-local Node.js script (`buildkite`) queries the Buildkite API for failed jobs and their logs. Locate it under the project's `.ai/skills/ci/` directory (typically a `.mjs` file). Run via `direnv exec .`. Requires `BUILDKITE_API_TOKEN` env var.

## Getting Failure Logs

1. Get the build URL: `gh pr checks --json name,state,link | jq -r '.[] | select(.state == "FAILURE") | .link'`
2. Parse org, pipeline, and build number from the URL (format: `https://buildkite.com/<org>/<pipeline>/builds/<number>...`)
3. List failed jobs: `direnv exec . <buildkite-script-path> failed <org> <pipeline> <build-number>`
   Each job includes `retried` (boolean) and `retried_in_job_id` fields.
4. If `failed` returns `[]` (empty): Buildkite auto-retries can mask failures. Re-query with `--include-retried`:
   `direnv exec . <buildkite-script-path> failed --include-retried <org> <pipeline> <build-number>`
   - If ALL returned jobs have `retried: true`: classify as **auto-retried flake** -- log job names, add check to handled_checks, continue.
   - If some jobs have `retried: false`: real failures. Fetch logs using `failed-logs --include-retried`.
   - If still empty after `--include-retried`: check `ci.pending` from poll response. If `ci.pending > 0`, the build is still in progress -- skip this check for the current iteration WITHOUT adding to `handled_checks` (re-evaluate next poll). Log: `- [<timestamp>] Deferred <check name>: failed/retried query empty but build still in progress (pending: <N>)`. Only classify as stale when `ci.pending == 0`: log it, add to handled_checks, continue.
5. Get logs: `direnv exec . <buildkite-script-path> failed-logs <org> <pipeline> <build-number>`
   If step 4's `--include-retried` fallback was triggered, use `failed-logs --include-retried`. Truncate to last 200 lines per job before passing to the fix subagent.

## Umbrella Check Handling

The umbrella parent check (e.g., `buildkite/gadget`) often stays in FAILURE state even after all child jobs pass on retry. If `exit` is null and the only failing check is the umbrella parent and all other actionable checks are passing, perform a final verification before declaring green:
- Run `<buildkite-script-path> failed <org> <pipeline> <build-number>` to confirm no failed jobs remain
- If `failed` returns results: real failure masked by shard overwriting -- dispatch for triage and fix, do NOT treat as all_green
- If `failed` returns empty: treat as effectively `all_green`

Detect umbrella-only condition: `ci.failed == 1` and the single failure name matches `^buildkite/[^/]+$` (two slash-separated components -- e.g., `buildkite/gadget` matches, `buildkite/gadget/node-api` does not). Add the umbrella check to handled_checks so it does not block future iterations.
