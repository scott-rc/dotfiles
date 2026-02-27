# Review

Fetch unresolved PR review threads and fix the issues reviewers described.

## Instructions

1. **Fetch unresolved review threads**:
   Run `get-pr-comments` (path in references/git-patterns.md).
   - If the script exits with an error (no PR exists), inform the user and stop.

2. **If no unresolved threads**, report that all review feedback has been addressed and stop.

3. **Present a summary to the user**:
   - Total count of unresolved threads
   - Group by file path, showing for each thread: file, line number, and a one-line preview of the first comment
   - Include any review summaries (these provide high-level context from the reviewer)
   - If many threads exist, group by file and show counts rather than listing every thread individually

4. **Gather context and fix each thread**:
   - Check the bulk threshold defined in references/bulk-threads.md. If below the threshold: read the relevant files inline at the indicated lines, read all comments in each thread (later replies often contain clarifications), and fix each thread directly.
   - If at or above the threshold: spawn an Explore subagent per references/bulk-threads.md (review variant) to gather per-thread context. Then fix each thread using the subagent's context summary.

   In both cases, group threads by file path to minimize context switching and apply the fix the reviewer requested.

5. **Verify fixes**: Run linter/tests if configured. Re-read changed code to confirm each thread is addressed. If any fix is incomplete, return to step 4 for that thread.

6. **After all fixes, offer follow-up actions** via AskUserQuestion: "Commit changes", "Commit and push", "Done (no commit)"
