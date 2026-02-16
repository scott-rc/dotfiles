# Review Operation

Fetch unresolved PR review threads and fix the issues reviewers described.

## Instructions

1. **Fetch unresolved review threads**:
   ```bash
   ~/.claude/skills/pr/scripts/get-pr-comments.sh
   ```
   - If the script exits with an error (no PR exists), inform the user and stop.

2. **If no unresolved threads**, report that all review feedback has been addressed and stop.

3. **Present a summary to the user**:
   - Total count of unresolved threads
   - Group by file path, showing for each thread: file, line number, and a one-line preview of the first comment
   - Include any review summaries (these provide high-level context from the reviewer)
   - If many threads exist, group by file and show counts rather than listing every thread individually

4. **Fix each unresolved thread**:
   - Read all comments in the thread -- later replies often contain clarifications or refined requests
   - Open the file at the indicated line
   - Understand the reviewer's concern and apply the fix
   - Group threads by file path to minimize context switching

5. **Verify fixes**: Run linter/tests if configured. Re-read changed code to confirm each thread is addressed. If any fix is incomplete, return to step 4 for that thread.

6. **After all fixes, offer follow-up actions**:
   - Commit the changes (git skill's commit operation)
   - Push to update the PR (git skill's push operation)
