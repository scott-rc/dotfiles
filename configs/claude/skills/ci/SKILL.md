---
name: ci
description: Checks GitHub Actions CI status, diagnoses failures from logs, fixes issues, and reruns jobs. Use when the user asks to check CI, check CI status, why is CI failing, fix CI, debug CI, rerun CI, retry CI, or re-trigger failed jobs.
---

# CI Operations

Route to the appropriate operation based on user intent.

## Operations

### Check
Check CI status for the current branch and summarize results.
See [check.md](check.md) for detailed instructions.

### Diagnose
Fetch failure logs, identify root cause, and fix the issues.
See [diagnose.md](diagnose.md) for detailed instructions.

### Rerun
Re-trigger failed CI jobs.
See [rerun.md](rerun.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"check CI"** / **"CI status"** → Run check operation
- **"why is CI failing"** / **"debug CI"** / **"fix CI"** → Run diagnose operation
- **"check and fix CI"** → Run check operation, then diagnose operation for any failures
- **"rerun CI"** / **"retry CI"** / **"re-trigger"** → Run rerun operation
- **"rerun and watch"** → Run rerun operation, then check operation to monitor new status

**Important**: You MUST read and follow the detailed operation file for each operation before executing it. Do not rely on the summaries above.

## Dependencies

Requires `gh` (GitHub CLI) and `git`.
