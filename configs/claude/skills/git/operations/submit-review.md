# Submit Review

Submit a PR review with a verdict (approve, request changes, or comment) and optional inline comments.

## Instructions

1. **Identify the PR**:
   - If the user provides a PR URL or number, extract owner/repo/number: `gh pr view {url_or_number} --json number,url`
   - If no PR specified, detect from current branch: `gh pr view --json number,url`
   - If no PR exists, inform the user and stop

2. **Determine the verdict**: Clarify the verdict before assembling the review payload.
   - `APPROVE` -- user says "approve", "lgtm", "looks good"
   - `REQUEST_CHANGES` -- user says "request changes", "block", "needs work"
   - `COMMENT` -- user says "comment only", or no explicit verdict
   - If ambiguous, ask the user

3. **Gather inline comments**: (Pre-delegation assembly — gathered after verdict because inline comment scope depends on review type.)
   If the user has review findings (from a prior code review, or described inline), map each to a review comment with these fields:
   - `path` -- file path relative to repo root
   - `line` -- line number in the file (MUST be within the PR diff)
   - `side` -- `RIGHT` for lines in the new file version (added or context lines), `LEFT` for removed lines. Default `RIGHT`.
   - `body` -- the comment text (MUST follow references/github-text.md)

   **Finding line numbers**: `line` is the file's line number, not a diff position. For comments on new/changed code, use the line number from the new version of the file. Read the file on the PR branch, or count from the diff hunk header's `+start` value (increment for each context and `+` line, skip `-` lines). The target line MUST appear in the diff -- GitHub rejects comments on lines outside the diff context.

4. **Collect review data**:
   Assemble the structured payload (do NOT write to a file -- the agent owns file I/O):

   - `event` -- `APPROVE`, `REQUEST_CHANGES`, or `COMMENT`
   - `body` -- optional top-level review summary (omit if inline comments suffice)
   - `comments` -- array of inline comments, each with `path`, `line`, `side`, `body` (can be empty for a plain approval)

   For multi-line comment ranges, add `start_line` and `start_side` to mark the range start.

5. **Submit the review**: Delegate to the `github-writer` agent with:
   - **type**: `review`
   - **body**: structured data -- `event`, optional `body`, and `comments` array
   - **target**: `owner`, `repo`, `pr_number`

6. **Report result**: Confirm the review was submitted with the verdict and number of inline comments posted.
