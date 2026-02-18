# Review Operation

Fetch unresolved PR review threads and fix the issues reviewers described.

## Instructions

1. **Fetch unresolved review threads**:
   ```bash
   ~/.claude/skills/git/scripts/get-pr-comments.sh
   ```
   - If the script exits with an error (no PR exists), inform the user and stop.

2. **If no unresolved threads**, report that all review feedback has been addressed and stop.

3. **Present a summary to the user**:
   - Total count of unresolved threads
   - Group by file path, showing for each thread: file, line number, and a one-line preview of the first comment
   - Include any review summaries (these provide high-level context from the reviewer)
   - If many threads exist, group by file and show counts rather than listing every thread individually

4. **Load coding preferences**: MUST read [general-guidelines.md](../code/general-guidelines.md). If fixing TypeScript/JavaScript files, also read [typescript-guidelines.md](../code/typescript-guidelines.md). Apply these preferences when writing fixes.

5. **Fix each unresolved thread**:
   - Read all comments in the thread -- later replies often contain clarifications or refined requests
   - Open the file at the indicated line
   - Understand the reviewer's concern and apply the fix
   - Group threads by file path to minimize context switching

6. **Verify fixes**: Run linter/tests if configured. Re-read changed code to confirm each thread is addressed. If any fix is incomplete, return to step 4 for that thread.

7. **If writing any text to GitHub** (PR comments, review replies, etc.): MUST follow the "All GitHub Text" section of [pr-guidelines.md](pr-guidelines.md) -- ASCII only, no em dashes, no curly quotes.

8. **After all fixes, offer follow-up actions**:
   - Commit the changes (commit operation)
   - Push to update the PR (push operation)
