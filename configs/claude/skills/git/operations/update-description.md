# Update Description

Rewrite the PR title and description to match current changes per guidelines.

## Instructions

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.
2. **Delegate to `pr-writer` agent** per references/pr-writer-rules.md with:
   - `mode`: `update`
   - `base_branch`: detect per references/git-patterns.md
   - `pr_number`: from step 1
   - `commit_messages`: read via `git log origin/<base>..HEAD --format=%B`
   - `branch_context` (optional): read `tmp/branches/<sanitized-branch>.md` if it exists (sanitize: replace `/` with `--`)
   - `context` (optional): one sentence if the user specified a reason for the update

   PR text MUST follow references/github-text.md.
3. **Check for unpushed history rewrite**: If the local HEAD differs from the remote tracking branch's HEAD (i.e., history was rewritten by a squash or amend but not yet pushed), present options via AskUserQuestion: "Force push (--force-with-lease)" or "Skip push". Only push if the user accepts.

4. **Report**: confirm update, show PR URL.
