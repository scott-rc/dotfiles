# CI Triage

How to classify and handle GitHub Actions CI failures. For Buildkite, see references/buildkite-handling.md.

## Fetching Failure Logs

```bash
gh run view <run_id> --log-failed 2>&1 | tail -300
```

Trim to relevant failure output: error messages, assertion diffs, stack traces, failing test names. Discard setup noise, progress bars, and timestamps. Focus on the first failure -- subsequent failures are often cascading.

**Fallback when the run is still in progress** (`--log-failed` exits non-zero or output indicates logs are unavailable):
- Fetch failed job details: `gh run view <run_id> --json jobs --jq '.jobs[] | select(.conclusion == "failure") | {name, conclusion}'`
- Fetch annotations: `gh run view <run_id>` -- scan for annotation lines with file paths, line numbers, and error descriptions
- Parse annotation lines for: file paths, line numbers, error descriptions
- Use collected job info and annotations as the "trimmed logs" for classification
- If failed jobs found but zero annotations parsed, report job names and conclusions with a note that full detail is pending. Proceed with classification using job name alone -- it often indicates the category (lint, test, build).

## Classification

### Step 1: Transient/Infrastructure?

Scan trimmed logs for these indicators:
- timeout, ETIMEDOUT
- connection refused, ECONNREFUSED
- rate limit, 429
- 503, 502, 504
- OOM, out of memory
- killed, signal 9
- runner lost
- no space left on device
- could not resolve host
- socket hang up

If found: rerun and report as transient.
```bash
gh run rerun <run_id> --failed
```

### Step 2: Flake?

Check if the same workflow has failed on the base branch within the last 7 days:
```bash
gh run list --branch <base_branch> --status failure --limit 5 --json databaseId,workflowName,createdAt
```

If a matching workflow name has a recent failure, classify as flake and rerun:
```bash
gh run rerun <run_id> --failed
```

Exception: if the failure is a snapshot mismatch, type error, or lint error that could result from a code change, classify as `real` even if the base branch has similar failures.

### Step 3: Real Failure

Extract actionable information:
- Test failures: failing test names and assertion messages
- Lint failures: file locations and specific errors
- Build failures: compilation error messages
