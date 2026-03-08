# Push

Push commits and create/update PR.

## Instructions

**Routing**: If the user asked to **refresh** or **update the PR description** without pushing new commits, skip to the **Refresh Description** mode at the end of this file.

1. **Check current branch**: Check main branch protection per references/git-patterns.md. If on a protected branch and the user declines to create a new branch, stop.

2. **Check for uncommitted changes**:
   - If changes exist, run the Commit operation first

3. **Push to remote**:
   - `git push -u origin HEAD`
   - If push is rejected (non-fast-forward), present options via AskUserQuestion: "Rebase onto remote", "Force push (--force-with-lease)", "Abort push"

4. **Check for existing PR** on this branch:
   ```bash
   gh pr view --json url,state,headRefOid 2>/dev/null
   ```

5. **Validate the PR is current** (not stale from an old branch with the same name):
   - If the PR's `state` is `MERGED` or `CLOSED`: treat as no PR exists (create a new one)
   - If the PR is `OPEN`, verify its head commit is in current history:
     - Check: `git merge-base --is-ancestor <headRefOid> HEAD`
     - If NOT an ancestor: present options via AskUserQuestion: "Close old PR and create new", "Abort push"

6. **Detect base branch and read context**: Detect base branch per references/git-patterns.md. Read the branch context file if it exists and does not contain the `N/A` sentinel (path and sentinel per references/git-patterns.md "Branch Context File"). Forward commit messages per the Commit Message Forwarding rule in references/pr-writer-rules.md.

7. **Create new PR**: If no PR exists (or old PR was merged/closed), and the dotfiles exception does not apply: if the branch context file is missing, run the Branch Context Creation pattern from `references/git-patterns.md` first. Then spawn `pr-writer` with `mode: create` using the Delegation Fields in references/pr-writer-rules.md.

8. **Update existing PR**: If a PR exists and new commits were pushed that aren't reflected in the current description: if the context file is somehow missing, run the Branch Context Creation pattern from `references/git-patterns.md` first. Then spawn `pr-writer` with `mode: update`. If no new commits were pushed (e.g., force push of same content), skip the update.

9. **Report PR URL** to the user.

---

## Refresh Description

Update the PR description without pushing new commits.

1. **Check for PR**: `gh pr view --json number,url,title,body 2>/dev/null`. If none, inform the user and stop.

2. **Ensure branch context**: Check if the branch context file exists (path per references/git-patterns.md "Branch Context File").
   - If **missing**: run the Branch Context Creation pattern from `references/git-patterns.md`.
   - If the file contains the `N/A` sentinel (per references/git-patterns.md "Opt-out sentinel") **and** the user did not already specify a reason for the update: ask via AskUserQuestion -- "What changed or why update the description?" with options:
     - **"I'll explain"** -- user provides the reason; use their response as the `context` field in the pr-writer delegation.
     - **"Just rewrite from the diff"** -- proceed without `context`.
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

6. **Report**: Confirm update, show PR URL.
