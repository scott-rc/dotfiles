# Update Description Operation

Rewrite the PR title and description to match the current changes per guidelines.

## Instructions

1. **Check for existing PR**:
   ```bash
   gh pr view --json number,url,title,body 2>/dev/null
   ```
   If no PR exists, inform the user and stop.

2. **Rewrite the PR title and description**:
   - You MUST follow [pr-guidelines.md](pr-guidelines.md) for the title and body
   - Run `git diff origin/<base>..HEAD` and `git diff --stat origin/<base>..HEAD` to understand the net change
   - Draft a new title and body based on the diff

3. **Preserve bot-appended content**:
   - Get current PR body: `gh pr view --json body -q .body`
   - If the PR body contains content not in your new description (appended by bots like Cursor BugBot, Dependabot):
     - Preserve that appended content
     - Update PR: `gh pr edit --title "<title>" --body "<new-body>\n\n<appended-content>"`

4. **Update the PR**:
   ```bash
   gh pr edit --title "<title>" --body "<body>"
   ```

5. **Report to user**: Confirm the PR was updated and show the PR URL.
