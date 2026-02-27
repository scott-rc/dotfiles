# Push

Push commits and create/update PR.

## Instructions

1. **Check current branch**:
   Check main branch protection per references/git-patterns.md. If dotfiles repo on main, push directly and **skip PR creation** (steps 4-8). If other repo on main/master, present branch options via AskUserQuestion; suggested branch names MUST follow the `sc/` prefix convention from references/git-patterns.md. If user stays on main, push directly and **skip PR creation**.

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

6. **If NO PR exists** (or old PR was merged/closed):
   Detect base branch per references/git-patterns.md. Spawn the `pr-writer` agent per references/pr-writer-rules.md with:
   - `mode`: `create`
   - `base_branch`: detected base branch
   - `context` (optional): one sentence of motivation -- the "why," not the "what"

   Example prompt: "mode: create, base_branch: main. Context: updates table references from v1 to v2 ahead of a follow-up drop migration."

7. **If PR exists and new commits were pushed that aren't reflected in the current description**:
   Detect base branch per references/git-patterns.md. Spawn the `pr-writer` agent per references/pr-writer-rules.md with:
   - `mode`: `update`
   - `base_branch`: detected base branch
   - `pr_number`: from step 4
   - `context` (optional): one sentence describing what changed since the last update

   If no new commits were pushed (e.g., force push of same content), skip the update.

   Example prompt: "mode: update, base_branch: main, pr_number: 123. Context: removed two tables from the migration after discovering they're still referenced elsewhere."

8. **Report PR URL** to the user. PR descriptions MUST follow references/github-text.md.
