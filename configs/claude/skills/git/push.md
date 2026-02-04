# Push Operation

Push commits and create/update PR.

## Instructions

1. **Check current branch**:
   - If on `main` or `master`:
     - For `dotfiles` repo: push directly to main and **skip PR creation** (steps 4-8)
     - Otherwise, ask the user if they want to create a new branch first
     - If the user chooses to stay on main, push directly and **skip PR creation** (steps 4-8)

2. Check for uncommitted changes:
   - If changes exist, commit them first (follow commit guidelines)

3. Push to remote:
   - `git push -u origin HEAD`

4. Check if a PR already exists for this branch:
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
   - Write the title and body following [pr-guidelines.md](pr-guidelines.md)

7. **If PR exists and description needs updating**:
   - Rewrite description following [pr-guidelines.md](pr-guidelines.md)
   - Get current PR body: `gh pr view --json body -q .body`
   - If PR body contains content not in your new description (appended by bots like Cursor BugBot, Dependabot):
     - Preserve that appended content
     - Update PR: `gh pr edit --title "<title>" --body "<new-body>\n\n<appended-content>"`

8. Report the PR URL to the user

See [git-patterns.md](git-patterns.md) for base branch detection and dotfiles exception patterns.
