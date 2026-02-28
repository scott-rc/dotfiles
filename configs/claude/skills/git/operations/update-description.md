# Update Description

Rewrite the PR title and description to match current changes per guidelines.

## Instructions

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.
2. **Delegate to `pr-writer` agent** per references/pr-writer-rules.md with:
   - `mode`: `update`
   - `base_branch`: detect per references/git-patterns.md
   - `pr_number`: from step 1
   - `commit_messages`: read via `git log origin/<base>..HEAD --format=%B`
   - `context` (optional): one sentence if the user specified a reason for the update

   PR text MUST follow references/github-text.md.
3. **Report**: confirm update, show PR URL.
