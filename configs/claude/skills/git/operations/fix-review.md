# Fix Review

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

4. **Classify threads by commenter type**: Use the Thread Classification rules in references/bulk-threads.md to determine which threads to fix autonomously (bots) and which require user approval (human reviewers).

5. **Gather context and fix each thread**: Agents MUST NOT post replies to human reviewer threads — only code fixes with user approval are permitted. Use references/bulk-threads.md for context gathering (Explore subagent threshold), then spawn a general-purpose subagent (model: sonnet) with:
   - Per-thread context: file path, line number(s), full comment bodies from all comments in each thread (later replies often contain clarifications), and the current code at those locations
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md, passed inline in the prompt
   - Instruction to: read the files at the referenced locations, apply the fix the reviewer requested, run the resolved lint and test commands, and consult the project's CLAUDE.md for project-specific build/test commands
   - Group threads by file path to minimize context switching; one subagent handles all threads in a batch

6. **Verify fixes**: Run linter/tests if configured per the "Local Fix Commands" section in references/git-patterns.md. Re-read changed code to confirm each thread is addressed. If any fix is incomplete after 2 attempts, report it as unresolved and continue with remaining threads. Report which threads were fixed and which files changed.

7. **Commit**: Stage changed files. Commit with message "Address PR review feedback: <brief summary of threads fixed>" per the Inline Commit Procedure in references/commit-message-format.md.

8. **Report**: Confirm what was fixed, which threads remain unresolved, and the commit hash.
