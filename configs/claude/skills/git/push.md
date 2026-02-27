# Push Operation

Push commits and create/update PR.

## Instructions

1. **Check current branch**:
   Check main branch protection per [git-patterns.md](references/git-patterns.md). If dotfiles repo on main, push directly and **skip PR creation** (steps 4-8). If other repo on main/master, present branch options via AskUserQuestion; suggested branch names MUST follow the `sc/` prefix convention from git-patterns.md. If user stays on main, push directly and **skip PR creation**.

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
   Detect base branch per [git-patterns.md](references/git-patterns.md). Spawn the `pr-writer` agent with:
   - `mode`: `create`
   - `base_branch`: detected base branch
   - `context` (optional): one sentence of motivation — the "why," not the "what"

   Do NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files in the prompt — the agent gathers its own diff and owns its own rules. If the agent fails, re-spawn it once — if it fails again, report the error to the user. Do NOT write the PR description yourself.

   Example prompt: "mode: create, base_branch: main. Context: updates table references from v1 to v2 ahead of a follow-up drop migration."

7. **If PR exists and new commits were pushed that aren't reflected in the current description**:
   Detect base branch per [git-patterns.md](references/git-patterns.md). Spawn the `pr-writer` agent with:
   - `mode`: `update`
   - `base_branch`: detected base branch
   - `pr_number`: from step 4
   - `context` (optional): one sentence describing what changed since the last update — the "why," not the "what"

   Do NOT include diff summaries, file lists, change descriptions, pre-drafted PR text, workflow commands, or references to skill/reference files in the prompt — the agent gathers its own diff and owns its own rules. If the agent fails, re-spawn it once — if it fails again, report the error to the user. Do NOT write the PR description yourself. If no new commits were pushed (e.g., force push of same content), skip the update.

   Example prompt: "mode: update, base_branch: main, pr_number: 123. Context: removed two tables from the migration after discovering they're still referenced elsewhere."

8. **Report PR URL** to the user

See [git-patterns.md](references/git-patterns.md) for base branch detection and dotfiles exception patterns.
