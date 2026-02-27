# Update Description

Rewrite the PR title and description to match current changes per guidelines.

## Instructions

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.
2. **Delegate to `pr-writer` agent** with:
   - `mode`: `update`
   - `base_branch`: detect per references/git-patterns.md
   - `pr_number`: from step 1
   - `context` (optional): one sentence if the user specified a reason for the update — the "why," not the "what"

   Do NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files in the prompt — the agent gathers its own diff and owns its own rules. If the agent fails, re-spawn it once — if it fails again, report the error to the user. Do NOT write the PR description yourself.
3. **Report**: confirm update, show PR URL.
