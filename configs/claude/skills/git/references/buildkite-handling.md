# Buildkite CI Handling

How to fetch logs and classify failures from Buildkite CI. Skip automated triage for Buildkite -- treat all non-flake failures as real.

## Overview

Buildkite runs sharded checks (e.g., ~80 parallel "node-api" jobs) that share the same check name in GitHub. A single failed shard gets masked by subsequent passing shards because GitHub reports the latest check result per name, not the worst. Whenever a Buildkite check appears in `gh pr checks` failed output — or whenever GitHub shows a Buildkite check as passing but you suspect shard masking — use the `buildkite` script to inspect the actual build.

The `buildkite` script is a project-local CI script located under `.ai/skills/ci/` (typically a `.mjs` file). Run it via `direnv exec . <path-to-script>`. Requires `BUILDKITE_API_TOKEN` env var.

Example invocation: `direnv exec . .ai/skills/ci/scripts/buildkite.mjs failed <org> <pipeline> <build>`. MUST NOT invoke via `node` — the script uses a zx shebang and requires direct execution.

## Getting Failure Logs

Build URL format: `https://buildkite.com/<org>/<pipeline>/builds/<number>...`. Obtain the URL via `gh pr checks --json name,state,link | jq -r '.[] | select(.state == "FAILURE") | .link'`.

The `buildkite` script's `failed` command lists failed jobs for a build, each with `retried` (boolean) and `retried_in_job_id` fields. The `failed-logs` command returns log output for all failed jobs; truncate to the last 200 lines per job before attempting fixes.

When `failed` returns an empty array, Buildkite auto-retries may have masked failures. The `--include-retried` flag re-queries including retried jobs. Possible outcomes:
- All returned jobs have `retried: true` — classify as auto-retried flake; log job names and treat as resolved.
- Some jobs have `retried: false` — real failures; fetch logs using `failed-logs --include-retried`.
- Still empty after `--include-retried` and the build is still running — skip this check for the current iteration without classifying it, and revisit on the next poll.
- Still empty after `--include-retried` and the build has finished — classify as stale and treat as resolved.

## Umbrella Check Handling

The umbrella parent check (e.g., `buildkite/gadget`) often stays in FAILURE state even after all child jobs pass on retry. If the only failing check is an umbrella parent and all other actionable checks are passing, perform a final verification before declaring the PR green:
- Run `<buildkite-script-path> failed <org> <pipeline> <build-number>` to confirm no failed jobs remain
- If `failed` returns results: real failure masked by shard overwriting — dispatch for triage and fix, do NOT treat as green
- If `failed` returns empty: treat the PR as effectively green

Detect umbrella-only condition: exactly one failing check and its name matches `^buildkite/[^/]+$` (two slash-separated components -- e.g., `buildkite/gadget` matches, `buildkite/gadget/node-api` does not).
