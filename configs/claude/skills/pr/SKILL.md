---
name: pr
description: Manages PR lifecycle -- checks CI status, diagnoses and reruns failed jobs, fetches and fixes PR review comments, and updates PR titles/descriptions. Use when the user asks to check CI, fix CI, rerun CI, address review comments, fix review feedback, fix bugbot comments, fix PR description, update PR, or sync PR.
---

# PR Operations

Route to the appropriate operation based on user intent.

## Operations

### Check
Check CI status for the current branch and summarize results.
See [check.md](check.md) for detailed instructions.

### Diagnose
Fetch CI failure logs, identify root cause, and fix the issues.
See [diagnose.md](diagnose.md) for detailed instructions.

### Rerun
Re-trigger failed CI jobs.
See [rerun.md](rerun.md) for detailed instructions.

### Review
Fetch unresolved PR review threads and fix the issues reviewers described.
See [review.md](review.md) for detailed instructions.

### Update Description
Rewrite the PR title and description to match current changes per guidelines.
See [update-description.md](update-description.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"check CI"** / **"CI status"** → Run check operation
- **"why is CI failing"** / **"debug CI"** / **"fix CI"** → Run diagnose operation
- **"check and fix CI"** → Run check operation, then diagnose operation for any failures
- **"rerun CI"** / **"retry CI"** / **"re-trigger"** → Run rerun operation
- **"rerun and watch"** → Run rerun operation, then check operation to monitor new status
- **"address review comments"** / **"fix review feedback"** / **"fix bugbot comments"** → Run review operation
- **"fix PR description"** / **"update PR"** / **"sync PR"** → Run update-description operation
- **"review and push"** / **"fix reviews and push"** → Run review operation, then use the git skill's push operation

**Important**: You MUST read and follow the detailed operation file for each operation before executing it. Do not rely on the summaries above.

## Dependencies

Requires `gh` (GitHub CLI), `git`, and `jq`.

## References

These files are referenced by the operation instructions above:

- [pr-guidelines.md](pr-guidelines.md) - Formatting rules for all GitHub-facing text (PR descriptions, comments, reviews)
