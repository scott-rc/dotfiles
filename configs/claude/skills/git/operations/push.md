# Push

Push commits and create/update PR.

## Instructions

**Routing**: If the user asked to **refresh** or **update the PR description** without pushing new commits, skip to the **Refresh Description** mode at the end of this file.

1. **Check current branch**: Check main branch protection per references/git-patterns.md. If on a protected branch and the user declines to create a new branch, stop.

2. **Check for uncommitted changes**:
   - If changes exist, run the Commit operation first

3. **Ensure git-spice**: Run the Ensure Git-Spice pattern from references/git-patterns.md.

4. **Detect stack push**: Run `git-spice log short 2>&1` to check if other branches in the stack also need pushing. For each branch listed, compare local vs remote: `git rev-list --left-right --count origin/<branch>...<branch>` — if the output shows commits on the left (remote has commits not in local) or the remote ref doesn't exist, the branch needs pushing. If multiple branches need pushing, use the **stack push** flow:
   - Run the Downstream PR Safety check from references/git-patterns.md for any branch that requires force push
   - Check if each branch in the stack has an existing PR: `gh pr view <branch> --json number,url 2>/dev/null`

   **If ALL branches already have PRs**: Use `git-spice stack submit --no-publish --no-prompt` (or `git-spice stack submit --no-publish --force --no-prompt` if any branch needs force push) to push all branches at once. For each branch with new commits, delegate to pr-writer with `mode: update`. Skip to the **Report** step.

   **If SOME or NO branches have PRs**: Use `git-spice stack submit --fill --no-prompt` (or `git-spice stack submit --fill --force --no-prompt` if any branch needs force push) to push all branches and create stub PRs with navigation comments for branches that lack them. Then for EACH branch in the stack:
   - If the branch just got a new PR (didn't have one before): check if the branch context file exists (path per references/git-patterns.md "Branch Context File"). If missing, check out the branch (`git-spice branch checkout <branch> --no-prompt`), run the Branch Context Creation pattern from references/git-patterns.md, then continue. Delegate to pr-writer with `mode: update` to replace the stub description.
   - If the branch already had a PR and received new commits: delegate to pr-writer with `mode: update`.
   - After processing all branches, check out back to the original branch if any checkout was performed.
   - After all pr-writer delegations complete, run the CR Discovery pattern from references/git-patterns.md (stack form) to ensure git-spice discovers the newly created PRs.

   Skip to the **Report** step after handling all branches.

   If only the current branch needs pushing (or not a git-spice stack), continue with the single-branch flow below.

5. **Push to remote**:
   - `git fetch origin`
   - **Detect divergence**: Run `git rev-list --left-right --count origin/$(git branch --show-current)...HEAD`. If the left count > 0, local history has diverged from remote (rebase, amend, or squash occurred) and force push is needed. If the remote tracking branch doesn't exist, this is a first push (no force needed).
   - **Regular push**: `git-spice branch submit --no-publish --no-prompt`
   - **Force push**: Run the Downstream PR Safety check from references/git-patterns.md first; after the user confirms via the safety check, use `git-spice branch submit --no-publish --force --no-prompt`.

6. **Check for existing PR** on this branch:
   ```bash
   gh pr view --json url,state,headRefOid 2>/dev/null
   ```

7. **Validate the PR is current** (not stale from an old branch with the same name):
   - If the PR's `state` is `MERGED` or `CLOSED`: treat as no PR exists (create a new one)
   - If the PR is `OPEN`, verify its head commit is in current history:
     - Check: `git merge-base --is-ancestor <headRefOid> HEAD`
     - If NOT an ancestor: present options via AskUserQuestion: "Close old PR and create new", "Abort push"

8. **Detect base branch and read context**: Detect base branch per references/git-patterns.md. Read the branch context file if it exists and does not contain the `N/A` sentinel (path and sentinel per references/git-patterns.md "Branch Context File"). Forward commit messages per the Commit Message Forwarding rule in references/pr-writer-rules.md.

9. **Context adequacy check**: Run `git diff --stat origin/<base>...HEAD` and count distinct top-level directories touched. If the diff touches 20+ files or spans 3+ distinct top-level directories AND the branch context is a single sentence (no line breaks, no bullets), the context may be stale or too thin. Present via AskUserQuestion: "The branch has grown since context was captured — update branch context?" with options:
   - **"Update it"** -- run the Branch Context Creation pattern (update path) from `references/git-patterns.md`, then continue.
   - **"Continue as-is"** -- proceed with existing context.

   Skip this check if the branch context file is missing (step 10/11 handles that) or contains the `N/A` sentinel.

10. **Create new PR**: If no PR exists (or old PR was merged/closed), and the dotfiles exception does not apply: if the branch context file is missing, run the Branch Context Creation pattern from `references/git-patterns.md` first (MUST follow the full pattern including the user confirmation step). Then spawn `pr-writer` with `mode: create` using the Delegation Fields in references/pr-writer-rules.md. After pr-writer completes, run the CR Discovery pattern from references/git-patterns.md (single-branch form) to ensure git-spice discovers the newly created PR.

11. **Update existing PR**: If a PR exists and new commits were pushed that aren't reflected in the current description: if the context file is somehow missing, run the Branch Context Creation pattern from `references/git-patterns.md` first (MUST follow the full pattern including the user confirmation step). Then spawn `pr-writer` with `mode: update`. If no new commits were pushed (e.g., force push of same content), skip the update.

12. **Report PR URL** to the user.

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

3. **Context adequacy check**: If the branch context file has real content (not missing, not `N/A`), detect base branch per references/git-patterns.md, then run `git diff --stat origin/<base>...HEAD` and count distinct top-level directories touched. If the diff touches 20+ files or spans 3+ distinct top-level directories AND the branch context is a single sentence (no line breaks, no bullets), present via AskUserQuestion: "The branch has grown since context was captured — update branch context?" with options:
   - **"Update it"** -- run the Branch Context Creation pattern (update path) from `references/git-patterns.md`, then continue.
   - **"Continue as-is"** -- proceed with existing context.

4. **Delegate to `pr-writer` agent** per references/pr-writer-rules.md with:
   - `mode`: `update`
   - `base_branch`: from step 3 (or detect per references/git-patterns.md if step 3 was skipped)
   - `pr_number`: from step 1
   - `commit_messages`: read via `git log origin/<base>..HEAD --format=%B`
   - `branch_context` (optional): read the branch context file if it exists and does not contain the `N/A` sentinel
   - `context` (optional): one sentence if the user specified a reason for the update (from step 2 or initial request)

   PR text MUST follow references/github-text.md.

5. **Check for unpushed history rewrite**: If the local HEAD differs from the remote tracking branch's HEAD (i.e., history was rewritten by a squash or amend but not yet pushed), present options via AskUserQuestion: "Force push (--force-with-lease)" or "Skip push". Only push if the user accepts.

6. **Verify**: Read back the posted description (`gh pr view <pr_number> --json body -q .body`). Spot-check any factual claims about before/after states (types, signatures, behavior changes) against the diff (re-read if needed). If something looks wrong, re-invoke the pr-writer with explicit correction context.

7. **Report**: Confirm update, show PR URL.
