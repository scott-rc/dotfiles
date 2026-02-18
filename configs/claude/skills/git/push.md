# Push Operation

Push commits and create/update PR.

## Instructions

1. **Check current branch**:
   - If on `main` or `master`:
     - For `dotfiles` repo: push directly to main and **skip PR creation** (steps 4-8)
     - Otherwise, ask the user if they want to create a new branch first
     - If the user chooses to stay on main, push directly and **skip PR creation** (steps 4-8)

2. **Check for uncommitted changes**:
   - If changes exist, commit them first (follow commit guidelines)

3. **Push to remote**:
   - `git push -u origin HEAD`
   - If push is rejected (non-fast-forward), offer to pull/rebase first or force push with `git push --force-with-lease`

4. **Check for existing PR** on this branch:
   ```bash
   gh pr view --json url,state,headRefOid 2>/dev/null
   ```

5. **Validate the PR is current** (not stale from an old branch with the same name):
   - If the PR's `state` is `MERGED` or `CLOSED`: treat as no PR exists (create a new one)
   - If the PR is `OPEN`, verify its head commit is in current history:
     - Check: `git merge-base --is-ancestor <headRefOid> HEAD`
     - If NOT an ancestor: ask the user if they want to close the old PR and create a new one, or abort

6. **If NO PR exists** (or old PR was merged/closed):
   - Create one with `gh pr create --title "<title>" --body "<body>"`
   - You MUST follow [pr-guidelines.md](pr-guidelines.md) for the title and body

7. **If PR exists and description needs updating**:
   - Follow the [Update Description operation](update-description.md) steps 2-5 to rewrite the title and description

8. **Report PR URL** to the user

See [git-patterns.md](git-patterns.md) for base branch detection and dotfiles exception patterns.
