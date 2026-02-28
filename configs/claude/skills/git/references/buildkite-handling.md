# Buildkite CI Handling

How to fetch logs and handle failures from Buildkite CI in the watch loop.

## Overview

Note: Buildkite runs sharded checks (e.g., ~80 parallel "node-api" jobs) that share the same check name in GitHub. A single failed shard gets masked by subsequent passing shards because GitHub reports the latest check result per name, not the worst. The `failed` count in the poll response correctly counts all failures, but `gh pr checks` may show the check as passing. Always use the `buildkite` script to inspect the actual build when the poll shows `failed > 0`, even if all named checks appear green.

The `buildkite` script is a project-local CI script located under `.ai/skills/ci/` (typically a `.mjs` file). Run it via `direnv exec . <path-to-script>`. Requires `BUILDKITE_API_TOKEN` env var.

## Getting Failure Logs

Build URL format: `https://buildkite.com/<org>/<pipeline>/builds/<number>...`. Obtain the URL via `gh pr checks --json name,state,link | jq -r '.[] | select(.state == "FAILURE") | .link'`.

The `buildkite` script's `failed` command lists failed jobs for a build, each with `retried` (boolean) and `retried_in_job_id` fields. The `failed-logs` command returns log output for all failed jobs; truncate to the last 200 lines per job before passing to a fix subagent.

When `failed` returns an empty array, Buildkite auto-retries may have masked failures. The `--include-retried` flag re-queries including retried jobs. Possible outcomes:
- All returned jobs have `retried: true` — classify as auto-retried flake; log job names, add check to `handled_checks`, continue.
- Some jobs have `retried: false` — real failures; fetch logs using `failed-logs --include-retried`.
- Still empty after `--include-retried` and `ci.pending > 0` — build still in progress; skip this check for the current iteration WITHOUT adding to `handled_checks`. Log: `- [<timestamp>] Deferred <check name>: failed/retried query empty but build still in progress (pending: <N>)`.
- Still empty after `--include-retried` and `ci.pending == 0` — classify as stale; log it, add to `handled_checks`, continue.

## Umbrella Check Handling

The umbrella parent check (e.g., `buildkite/gadget`) often stays in FAILURE state even after all child jobs pass on retry. If `exit` is null and the only failing check is the umbrella parent and all other actionable checks are passing, perform a final verification before declaring green:
- Run `<buildkite-script-path> failed <org> <pipeline> <build-number>` to confirm no failed jobs remain
- If `failed` returns results: real failure masked by shard overwriting -- dispatch for triage and fix, do NOT treat as all_green
- If `failed` returns empty: treat as effectively `all_green`

Detect umbrella-only condition: `ci.failed == 1` and the single failure name matches `^buildkite/[^/]+$` (two slash-separated components -- e.g., `buildkite/gadget` matches, `buildkite/gadget/node-api` does not). Add the umbrella check to handled_checks so it does not block future iterations.
