# Push Operation

Push commits and create/update PR.

## Instructions

1. **Check current branch**:
   Check main branch protection per [git-patterns.md](git-patterns.md). If dotfiles repo on main, push directly and **skip PR creation** (steps 4-8). If other repo on main/master, present branch options via AskUserQuestion; if user stays on main, push directly and **skip PR creation**.

2. **Check for uncommitted changes**:
   - If changes exist, run the Commit operation first

3. **Push to remote**:
   - `git push -u origin HEAD`
   - If push is rejected (non-fast-forward), present options via AskUserQuestion: "Rebase onto remote", "Force push (--force-with-lease)"

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
   Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `pr-writer` agent with: mode `create`, base_branch.

7. **If PR exists and description needs updating**:
   Detect base branch per [git-patterns.md](git-patterns.md). Spawn the `pr-writer` agent with: mode `update`, base_branch, pr_number.

8. **Report PR URL** to the user

See [git-patterns.md](git-patterns.md) for base branch detection and dotfiles exception patterns.
