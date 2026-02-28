# Push

Push commits and create/update PR.

## Instructions

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

6. **If NO PR exists** (or old PR was merged/closed):
   Detect base branch per references/git-patterns.md. Read all branch commit messages: `git log origin/<base>..HEAD --format=%B`. Spawn the `pr-writer` agent per references/pr-writer-rules.md with:
   - `mode`: `create`
   - `base_branch`: detected base branch
   - `commit_messages`: all branch commit messages verbatim
   - `context` (optional): one sentence of motivation -- the "why," not the "what"
   - PR text MUST follow references/github-text.md.

7. **If PR exists and new commits were pushed that aren't reflected in the current description**:
   Detect base branch per references/git-patterns.md. Read all branch commit messages: `git log origin/<base>..HEAD --format=%B`. Spawn the `pr-writer` agent per references/pr-writer-rules.md with:
   - `mode`: `update`
   - `base_branch`: detected base branch
   - `pr_number`: from step 4
   - `commit_messages`: all branch commit messages verbatim
   - `context` (optional): one sentence describing what changed since the last update
   - PR text MUST follow references/github-text.md.

   If no new commits were pushed (e.g., force push of same content), skip the update.

8. **Report PR URL** to the user.
