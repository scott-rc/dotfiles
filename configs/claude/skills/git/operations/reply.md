# Reply

Fetch unreplied PR review threads and draft responses for user approval, or post a specific comment on a GitHub PR or issue.

## Instructions

### Auto-discover mode (no specific target provided)

1. **Fetch unreplied threads**:
   Run `get-pr-comments --unreplied` (path in references/git-patterns.md).
   If the script exits with an error (no PR exists), inform the user and stop.

2. **If no unreplied threads**, report that all review feedback has been responded to and stop.

3. **Present a summary**: Total count of unreplied threads, grouped by file path with line number and a one-line preview of each reviewer comment. Include review summaries for high-level context.

4. **Gather context and draft replies**:
   - Check the bulk threshold defined in references/bulk-threads.md. If below the threshold: read all comments in each thread inline (later replies often contain clarifications), check `git diff` for relevant files, and draft replies directly.
   - If at or above the threshold: spawn an Explore subagent per references/bulk-threads.md (reply variant) to gather per-thread context. Then draft replies using the subagent's context summary.

   For each reply:
   - If code was changed to address the feedback, reference what was done
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

All posted text MUST follow references/github-text.md. Delegate to the `github-writer` agent for each approved reply. Provide:

- **type**: `review-reply` for review threads, `pr-comment` for PR comments, `issue-comment` for issue comments
- **body**: the approved reply text
- **target**: the relevant identifiers (`owner`, `repo`, `comment_id` for review replies; `pr_number` for PR comments; `issue_number` for issue comments)
