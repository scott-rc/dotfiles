# Reply Operation

Fetch unreplied PR review threads and draft responses for user approval, or post a specific comment on a GitHub PR or issue.

## Instructions

### Auto-discover mode (no specific target provided)

1. **Fetch unreplied threads**:
   ```bash
   ~/.claude/skills/git/scripts/get-unreplied-comments.sh
   ```
   If the script exits with an error (no PR exists), inform the user and stop.

2. **If no unreplied threads**, report that all review feedback has been responded to and stop.

3. **Present a summary**: Total count of unreplied threads, grouped by file path with line number and a one-line preview of each reviewer comment. Include review summaries for high-level context.

4. **Draft replies for each thread**:
   When there are many threads (5+), spawn a Task subagent (type: Explore, model: sonnet) to read all referenced files and check `git diff` for relevant changes, returning a per-thread summary of the current code state and any changes made. Use this context to draft replies without reading each file inline.

   For each thread:
   - Read all comments in the thread -- later replies often contain clarifications
   - If code was changed to address the feedback (use the subagent's diff analysis or check `git diff` for relevant files), reference what was done
   - If the feedback was already addressed, say so concisely
   - If the feedback needs discussion, draft a thoughtful response

5. **Present ALL drafts for user review**: Show each draft alongside the reviewer's comment for context. For each draft, present options via AskUserQuestion: "Approve", "Skip", "Edit". MUST wait for user approval before posting anything.

6. **Post approved replies** (see Posting below).

7. **Report summary**: Confirm which replies were posted and which were skipped.

### Specific target mode (URL, comment, or issue provided)

1. **Draft the reply** based on user instructions.

2. **Present draft for user review**: MUST show the draft and wait for approval before posting.

3. **Post** (see Posting below).

### Posting

MUST follow the "All GitHub Text" section of [pr-guidelines.md](pr-guidelines.md) -- ASCII only, no em dashes, no curly quotes.

Write body to a temp file and post using `-F body=@file` to avoid shell encoding issues:

```bash
# Reply to a review comment thread (use last comment's id from script output)
echo 'content' > /tmp/gh-comment-body.txt
gh api repos/{owner}/{repo}/pulls/comments/{comment_id}/replies -F body=@/tmp/gh-comment-body.txt

# Comment on a PR
echo 'content' > /tmp/gh-comment-body.txt
gh pr comment {pr_number} -F /tmp/gh-comment-body.txt

# Comment on an issue
echo 'content' > /tmp/gh-comment-body.txt
gh issue comment {issue_number} -F /tmp/gh-comment-body.txt
```

Clean up the temp file after posting.
