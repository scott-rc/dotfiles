# Push

Push commits and create/update PR.

## Instructions

**Routing**: If the user asked to **refresh** or **update the PR description** without pushing new commits, skip to the **Refresh Description** mode at the end of this file.

1. **Check current branch**: Check main branch protection per references/git-patterns.md. If on a protected branch and the user declines to create a new branch, stop.

2. **Check for uncommitted changes**:
   - If changes exist, run the Commit operation first

3. **Gather stack metadata**: Run `git-spice log short --json 2>/dev/null` to collect all stack metadata in one pass.
   - If the command fails: follow the Error Recovery pattern from references/git-spice-patterns.md, then retry.
   - Parse the JSONL output per the "Stack Metadata via JSON" section of references/git-spice-patterns.md. For each line: `.name` is the branch name, `.change` holds PR info (null if no PR), `.push.ahead` and `.push.behind` indicate divergence vs remote, `.down.name` is the base branch.
   - From the current branch's entry: note whether `.change` is non-null (PR exists) and extract `.push.ahead`/`.push.behind`.
   - Identify branches needing pushing: any branch where `.push.ahead > 0`, `.push.behind > 0`, or `.push` is absent (remote doesn't exist).

4. **Detect stack push**: If multiple branches need pushing (from step 3), use the **stack push** flow:
   - PR existence per branch comes from `.change != null` in the JSON — no need for per-branch `gh pr view` calls.
   - Run the Downstream PR Safety check from references/git-patterns.md for any branch where `.push.behind > 0`.

   **If ALL branches already have PRs** (all `.change != null`): Use `git-spice stack submit --update-only --no-prompt` (or `git-spice stack submit --update-only --force --no-prompt` if any branch has `.push.behind > 0`) to push all branches at once. For each branch with new commits, write an updated PR title and description inline following references/pr-writer-rules.md. Skip to the **Report** step.

   **If SOME or NO branches have PRs**: Use `git-spice stack submit --fill --no-prompt` (or `git-spice stack submit --fill --force --no-prompt` if any branch has `.push.behind > 0`) to push all branches and create stub PRs with navigation comments for branches that lack them. Then for EACH branch in the stack:
   - If the branch just got a new PR (`.change` was null before submit): check if the branch context file exists (path per references/git-patterns.md "Branch Context File"). If missing, check out the branch (`git-spice branch checkout <branch> --no-prompt`), run the Branch Context Creation pattern from references/git-patterns.md, then continue. Write an updated PR title and description inline following references/pr-writer-rules.md to replace the stub description.
   - If the branch already had a PR and received new commits: write an updated PR title and description inline following references/pr-writer-rules.md.
   - After processing all branches, check out back to the original branch if any checkout was performed.

   Skip to the **Report** step after handling all branches.

   If only the current branch needs pushing (or not a git-spice stack), continue with the single-branch flow below.

5. **Push to remote**:
   - `git fetch origin`
   - **Detect divergence**: `.push.behind > 0` from step 3 reflects pre-fetch state and is useful for anticipating whether a force push may be needed (e.g., preparing the user). After `git fetch origin`, run `git rev-list --left-right --count origin/$(git branch --show-current)...HEAD` for the authoritative post-fetch divergence state — fetch can change the picture. Left count > 0 means force push is needed. If the remote tracking branch doesn't exist, this is a first push (no force needed).
   - **Select flag**: Use `--update-only` if the branch has a PR (`.change` is non-null from step 3); use `--no-publish` if it does not. See the Push via Git-Spice pattern in references/git-spice-patterns.md for flag details.
   - **Regular push**: `git-spice branch submit <flag> --no-prompt`
   - **Force push**: Run the Downstream PR Safety check from references/git-patterns.md first; after the user confirms, use `git-spice branch submit <flag> --force --no-prompt`.

6. **Validate PR**: Use `.change` from step 3 to determine next action:
   - If `.change` is null: no PR exists — skip to step 9 (create PR).
   - If `.change` is non-null: run `gh pr view --json state,headRefOid 2>/dev/null` (PR number known from `.change.id`) to check for staleness:
     - If `state` is `MERGED` or `CLOSED`: treat as no PR (skip to step 9).
     - If `state` is `OPEN`, verify head commit: `git merge-base --is-ancestor <headRefOid> HEAD`. If NOT an ancestor: present options via AskUserQuestion: "Close old PR and create new", "Abort push".

7. **Gather PR context** (run in parallel after push):
   - Read branch context file (path per references/git-patterns.md "Branch Context File")
   - `git log origin/<base>..HEAD --format=%B` for commit messages (base branch from `.down.name` in step 3 JSON)
   - `git diff --stat origin/<base>...HEAD` for diff stats

8. **Context adequacy check**: Count distinct top-level directories from the diff stats. If 20+ files or 3+ distinct top-level directories AND the branch context is a single sentence (no line breaks, no bullets), the context may be stale or thin. Present via AskUserQuestion: "The branch has grown since context was captured — update branch context?" with options:
   - **"Update it"** -- run the Branch Context Creation pattern (update path) from references/git-patterns.md, then continue.
   - **"Continue as-is"** -- proceed with existing context.

   Skip this check if the branch context file is missing (step 9/10 handles that) or contains the `N/A` sentinel.

9. **Create new PR**: If no PR exists (or old PR was merged/closed), and the dotfiles exception does not apply: if the branch context file is missing, run the Branch Context Creation pattern from references/git-patterns.md first (MUST follow the full pattern including the user confirmation step). Then write the PR title and description inline following references/pr-writer-rules.md and create the PR.

10. **Update existing PR**: If a PR exists and new commits were pushed that aren't reflected in the current description: if the context file is somehow missing, run the Branch Context Creation pattern from references/git-patterns.md first (MUST follow the full pattern including the user confirmation step). Then write an updated PR title and description inline following references/pr-writer-rules.md. If no new commits were pushed (e.g., force push of same content), skip the update.

11. **Report PR URL** to the user.

---

## Refresh Description

Update the PR description without pushing new commits.

1. **Check for PR**: Read stack metadata via the Stack Metadata via JSON pattern in references/git-spice-patterns.md. Check the current branch's `.change` field. If `.change` is null, inform the user that no PR exists and stop. Extract PR number from `.change.id` (strip the `#` prefix) and URL from `.change.url`.

2. **Ensure branch context**: Check if the branch context file exists (path per references/git-patterns.md "Branch Context File").
   - If **missing**: run the Branch Context Creation pattern from references/git-patterns.md.
   - If the file contains the `N/A` sentinel (per references/git-patterns.md "Opt-out sentinel") **and** the user did not already specify a reason for the update: ask via AskUserQuestion -- "What changed or why update the description?" with options:
     - **"I'll explain"** -- user provides the reason; use their response as the `context` field when writing the PR description.
     - **"Just rewrite from the diff"** -- proceed without `context`.
   - If the file has real content: proceed normally (`branch_context` carries the motivation).

3. **Context adequacy check**: If the branch context file has real content (not missing, not `N/A`), detect base branch per references/git-patterns.md, then run `git diff --stat origin/<base>...HEAD` and count distinct top-level directories touched. If the diff touches 20+ files or spans 3+ distinct top-level directories AND the branch context is a single sentence (no line breaks, no bullets), present via AskUserQuestion: "The branch has grown since context was captured — update branch context?" with options:
   - **"Update it"** -- run the Branch Context Creation pattern (update path) from references/git-patterns.md, then continue.
   - **"Continue as-is"** -- proceed with existing context.

4. **Write PR title and description inline** following references/pr-writer-rules.md. Gather the required context:
   - `base_branch`: from step 3 (or detect per references/git-patterns.md if step 3 was skipped)
   - `pr_number`: from step 1
   - `commit_messages`: read via `git log origin/<base>..HEAD --format=%B`
   - `branch_context` (optional): read the branch context file if it exists and does not contain the `N/A` sentinel
   - `context` (optional): one sentence if the user specified a reason for the update (from step 2 or initial request)

   PR text MUST follow references/github-text.md.

5. **Check for unpushed history rewrite**: If the local HEAD differs from the remote tracking branch's HEAD (i.e., history was rewritten by a squash or amend but not yet pushed), first run the Downstream PR Safety check from references/git-patterns.md, then present options via AskUserQuestion: "Force push" or "Skip push". Only push if the user accepts. To push: check PR existence via the Stack Metadata via JSON pattern in references/git-spice-patterns.md (`.change` field). If the branch has a PR, use `git-spice branch submit --update-only --force --no-prompt`; if no PR, use `git-spice branch submit --no-publish --force --no-prompt`.

6. **Verify**: Read back the posted description (`gh pr view <pr_number> --json body -q .body`). Spot-check any factual claims about before/after states (types, signatures, behavior changes) against the diff (re-read if needed). If something looks wrong, correct the description inline and re-post.

7. **Report**: Confirm update, show PR URL.
