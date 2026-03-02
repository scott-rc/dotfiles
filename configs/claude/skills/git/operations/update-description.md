# Update Description

Rewrite the PR title and description to match current changes per guidelines.

## Instructions

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform user and stop.

2. **Ensure branch context**: Check if the branch context file exists (path per references/git-patterns.md "Branch Context File").
   - If **missing**: run the set-branch-context operation first.
   - If the file contains the `N/A` sentinel (per references/git-patterns.md "Opt-out sentinel") **and** the user did not already specify a reason for the update: ask via AskUserQuestion -- "What changed or why update the description?" with options:
     - **"I'll explain"** — user provides the reason; use their response as the `context` field in the pr-writer delegation.
     - **"Just rewrite from the diff"** — proceed without `context`.
   - If the file has real content: proceed normally (`branch_context` carries the motivation).

3. **Delegate to `pr-writer` agent** per references/pr-writer-rules.md with:
   - `mode`: `update`
   - `base_branch`: detect per references/git-patterns.md
   - `pr_number`: from step 1
   - `commit_messages`: read via `git log origin/<base>..HEAD --format=%B`
   - `branch_context` (optional): read the branch context file if it exists and does not contain the `N/A` sentinel
   - `context` (optional): one sentence if the user specified a reason for the update (from step 2 or initial request)

   PR text MUST follow references/github-text.md.

4. **Check for unpushed history rewrite**: If the local HEAD differs from the remote tracking branch's HEAD (i.e., history was rewritten by a squash or amend but not yet pushed), present options via AskUserQuestion: "Force push (--force-with-lease)" or "Skip push". Only push if the user accepts.

5. **Verify**: Read back the posted description (`gh pr view <pr_number> --json body -q .body`). Spot-check any factual claims about before/after states (types, signatures, behavior changes) against the diff (re-read if needed). If something looks wrong, re-invoke the pr-writer with explicit correction context.

6. **Report**: confirm update, show PR URL.
