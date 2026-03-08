# Fix

Auto-detect and fix CI failures and unresolved review threads; draft and post replies.

## Instructions

### Detection

Run these two checks in parallel:

1. **Review threads**: Run `~/.claude/skills/git/scripts/get-pr-comments.sh` to get unresolved thread count and thread list.
2. **CI status**: Follow the "CI Detection" pattern in references/git-patterns.md (steps 1 and 2). Group checks by status (failed, pending, passed).

After both complete, route based on results:

- **CI failures only** (no unresolved threads): follow the CI path below.
- **Unresolved threads only** (CI green or no CI): follow the Review path below.
- **Both CI failures and unresolved threads**: follow the Combined path below.
- **Neither** (CI green and no unresolved threads): report that everything is green and stop.

---

### CI Path

1. **Detect CI system** per the "CI System Detection" section in references/git-patterns.md: check `.github/workflows/` for github-actions, `.buildkite/` for buildkite.

   - **GitHub Actions**: Use `~/.claude/skills/git/scripts/get-failed-runs.sh` to get failed run IDs and workflow names. If the script returns an empty array but detection found failures, runs may still be initializing -- report to the user and suggest retrying shortly. Detect base branch per references/git-patterns.md. Spawn a `ci-triager` agent for each distinct failed run with: run_id, workflow_name, branch, base_branch, and repo.

     Based on the triager's classification:
     - **transient** or **flake**: report the classification to the user (the triager already reran the job). Stop.
     - **real**: proceed to the fix step using the trimmed logs and root cause from the triager's report.

   - **Buildkite**: Skip triaging. Fetch logs per references/buildkite-handling.md. MUST NOT use `ci-triager` for Buildkite -- treat all failures as real. Truncate logs to the last 200 lines per job.

2. **Fix the issues**: Spawn a general-purpose subagent (model: sonnet) with:
   - For GitHub Actions: the triager's full report as task context (root cause analysis, trimmed failure logs, relevant file paths)
   - For Buildkite: the truncated failure logs
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md, passed inline in the prompt
   - Instruction to: read the source files identified in the report/logs, apply the fix, run the resolved lint and test commands, and consult the project's CLAUDE.md for project-specific build/test commands
   - One subagent per failed check

   If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options before accepting the subagent's changes. If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user via AskUserQuestion before applying.

3. **Report and commit**: Summarize what failed, why, what was fixed, and whether local verification passed. Offer to commit and push. If the user accepts: stage changed files, commit with message "Fix <workflow/check name> CI failure: <brief cause>" per the Inline Commit Procedure in references/commit-message-format.md, then run the Push operation (push's uncommitted-changes check is redundant after a fresh commit -- skip it).

---

### Review Path

1. **Check for PR**: If `get-pr-comments` exited with an error (no PR exists), inform the user and stop.

2. **If no unresolved threads**, report that all review feedback has been addressed and stop.

3. **Present a summary**:
   - Total count of unresolved threads
   - Group by file path, showing for each thread: file, line number, and a one-line preview of the first comment
   - Include any review summaries (these provide high-level context from the reviewer)
   - If many threads exist, group by file and show counts rather than listing every thread individually

4. **Classify threads by commenter type**: Use the Thread Classification rules in references/bulk-threads.md to determine which threads to fix autonomously (bots) and which require user approval (human reviewers). Agents MUST NOT post replies to human reviewer threads. MUST ask the user to confirm via AskUserQuestion before applying code fixes to human threads.

5. **Gather context and fix**: Check the bulk threshold in references/bulk-threads.md (>=5 threads OR <5 touching >3 files). If at or above the threshold: spawn an Explore subagent (model: sonnet) to read each referenced file with 10-20 lines of surrounding context at each thread location and return a per-thread summary. Then spawn a fix subagent (model: sonnet) with:
   - Per-thread context: file path, line number(s), full comment bodies from all comments in each thread (later replies often contain clarifications), and the current code at those locations
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md, passed inline in the prompt
   - Instruction to: read the files at the referenced locations, apply the fix the reviewer requested, run the resolved lint and test commands, and consult the project's CLAUDE.md for project-specific build/test commands
   - Group threads by file path to minimize context switching; one subagent handles all threads

6. **Verify fixes**: Run linter/tests per the "Local Fix Commands" section in references/git-patterns.md. Re-read changed code to confirm each thread is addressed. If any fix is incomplete after 2 attempts, mark it as unresolved and continue with remaining threads.

7. **Commit**: Stage changed files. Commit with message "Address PR review feedback: <brief summary of threads fixed>" per the Inline Commit Procedure in references/commit-message-format.md.

8. **Draft replies**: After code fixes are committed, draft a reply for each thread:
   - If code was changed to address the feedback, reference what was done
   - If the feedback was already addressed, say so concisely
   - If the feedback needs discussion, draft a thoughtful response

9. **Present drafts for approval**: Show each draft alongside the reviewer's comment for context. For each draft, present options via AskUserQuestion: "Approve", "Skip", "Edit". MUST NOT post any reply to a human reviewer's comment without showing the draft and receiving explicit user approval.

10. **Post approved replies**: All posted text MUST follow references/github-text.md. For each approved reply, write the body to a temp file, validate ASCII with `~/.claude/skills/git/scripts/safe-text.sh`, and post:
    - Review reply: `gh api repos/{owner}/{repo}/pulls/comments/{comment_id}/replies -F body=@"$TMPFILE"`
    - PR comment: `gh pr comment {pr_number} --repo {owner}/{repo} --body-file "$TMPFILE"`
    Clean up temp files after posting.

11. **Report**: Confirm which threads were fixed, which replies were posted, which were skipped, and which remain unresolved.

---

### Combined Path

When both CI failures and unresolved review threads exist:

1. Run the **CI path** first (through commit).
2. Run the **Review path** (all steps including replies).
3. Report a unified summary: CI fixes applied, review fixes applied, replies posted, and any unresolved items.
