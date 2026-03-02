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

6. **Detect base branch and read context**: Detect base branch per references/git-patterns.md. Read the branch context file if it exists and does not contain the `N/A` sentinel (path and sentinel per references/git-patterns.md "Branch Context File"). Forward commit messages per the Commit Message Forwarding rule in references/pr-writer-rules.md.

7. **Create new PR**: If no PR exists (or old PR was merged/closed), and the dotfiles exception does not apply: if the branch context file is missing, run the set-branch-context operation first. Then spawn `pr-writer` with `mode: create` using the Delegation Fields in references/pr-writer-rules.md.

8. **Update existing PR**: If a PR exists and new commits were pushed that aren't reflected in the current description: if the context file is somehow missing, run set-branch-context first. Then spawn `pr-writer` with `mode: update`. If no new commits were pushed (e.g., force push of same content), skip the update.

9. **Report PR URL** to the user.
