# Fix

Auto-detect and fix CI failures, unresolved review threads, and PR description quality issues.

## Instructions

### Detection

Run these three checks in parallel:

User-provided instructions (e.g., "fix and ensure contributing is up to date") supplement but do not replace Detection. MUST run all three checks regardless of additional context. Handle side-requests as a separate concern after the standard fix paths complete.

1. **Review threads**: Run `~/.claude/skills/git/scripts/get-pr-comments.sh --count` to get the unresolved thread count.
2. **CI status**: Follow the "CI Detection" pattern in references/git-patterns.md (steps 1 and 2). Group checks by status (failed, pending, passed).
3. **PR description quality**: Fetch the current PR body (`gh pr view --json body,number -q '.body'`). If no PR exists, skip. Detect base branch per references/git-patterns.md. Run `git diff --stat origin/<base>...HEAD` and assess:

   The description needs a refresh if ANY of:
   - **Wall of text**: The diff spans 3+ distinct top-level directories (proxy for multiple concerns) but the body contains no numbered lists, no bullet lists, and no blank-line-separated short paragraphs (i.e., it's 2+ dense paragraphs).
   - **Missing diff coverage**: The `--stat` output shows file groups (e.g., `.claude/`, `.github/`, `docs/`, test files) that the body doesn't mention at all -- not even in passing.
   - **Missing verification info**: The diff includes CI workflow files (`.github/workflows/`, `.buildkite/`), deploy configs, or build system changes, but the body says nothing about how to verify locally or what CI triggers.

   If none of these trigger, mark the description as OK. Note: review threads that flag description inaccuracy are handled by Review Path step 3 classification, not here.

After all three complete, route based on results:

- **CI failures only** (no unresolved threads, description OK): follow the CI path below.
- **Unresolved threads only** (CI green or no CI, description OK): follow the Review path below.
- **Both CI failures and unresolved threads** (description OK): follow the Combined path below.
- **Description issues only** (CI green, no unresolved threads): follow the Description path below.
- **Neither** (CI green, no unresolved threads, description OK): report that everything is green. Then run `CronList` — if any job's prompt contains `/git fix`, cancel it with `CronDelete` and tell the user the loop has stopped because all checks passed and all threads are resolved. This is safe because cron jobs are session-scoped; parallel loops on other branches run in separate sessions.

If CI, Review, or Combined path ran AND the description quality check (wall of text, missing coverage, or missing verification info) also flagged issues, run the Description path after all other paths complete. Description threads found by Review Path step 3 are handled by Review Path step 8 — they do not trigger this rule.

---

### CI Path

1. **Detect CI system** per the "CI System Detection" section in references/git-patterns.md: check `.github/workflows/` for github-actions, `.buildkite/` for buildkite.

   - **GitHub Actions**: Use `~/.claude/skills/git/scripts/get-failed-runs.sh` to get failed run IDs and workflow names. If the script returns an empty array but detection found failures, runs may still be initializing -- report to the user and suggest retrying shortly. Detect base branch per references/git-patterns.md. Spawn a `ci-triager` agent for each distinct failed run with: run_id, workflow_name, branch, base_branch, and repo.

     Based on the triager's classification:
     - **transient** or **flake**: report the classification to the user (the triager already reran the job). Stop.
     - **real**: proceed to the fix step using the trimmed logs and root cause from the triager's report.

   - **Buildkite**: Fetch logs per references/buildkite-handling.md. MUST NOT use `ci-triager` for Buildkite. Truncate logs to the last 200 lines per job. **Before dispatching a fix subagent**, present a summary of failed jobs (job name + one-line failure snippet from each log) and ask the user via AskUserQuestion with options: "Fix all", "Skip (likely flakes)", "Let me choose". If "Let me choose", present each job individually and let the user select which to fix. Only dispatch fixes for jobs the user selected.

2. **Fix the issues**: Spawn a `code-writer` subagent (model: sonnet) with:
   - For GitHub Actions: the triager's full report as task context (root cause analysis, trimmed failure logs, relevant file paths)
   - For Buildkite: the truncated failure logs
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md, passed inline in the prompt as constraints
   - Reference to the project's CLAUDE.md for project-specific build/test commands
   - One subagent per failed GitHub Actions run; a single subagent handles all Buildkite failures

   If the fix is ambiguous or risky, present candidate fixes as AskUserQuestion options before accepting the subagent's changes. If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user via AskUserQuestion before applying.

3. **Commit and push**: Summarize what failed, why, what was fixed, and whether local verification passed. Stage changed files, commit with message "Fix <workflow/check name> CI failure: <brief cause>" per the Inline Commit Procedure in references/commit-message-format.md, then push with `git push`.

---

### Review Path

1. **Check for PR**: If Detection step 1 reported no PR, stop.

2. **Present a summary**: Run `get-pr-comments.sh --summary` and present its compact output. For large thread sets, group by file and show counts rather than listing every thread individually.

3. **Classify threads**: Use the Thread Classification rules in references/bulk-threads.md to determine which threads to fix autonomously (bots) and which require user approval (human reviewers). Agents MUST NOT post replies to human reviewer threads. MUST ask the user to confirm via AskUserQuestion before applying code fixes to human threads.

   Independently classify each thread by **target**: **code threads** (feedback about source code, tests, configs, or CI files) vs **description threads** (feedback about the PR title, summary, or description text being inaccurate, misleading, or incomplete — the comment discusses what the PR *says*, not what it *does*). Classify each thread on both axes independently — a thread may be both a bot thread and a description thread.

4. **Gather context and fix**: This step applies to **code threads** only. Skip if all threads are description threads.

   Check the bulk threshold in references/bulk-threads.md (>=5 threads OR <5 touching >3 files). If at or above the threshold: spawn an Explore subagent (model: sonnet) to read each referenced file with 10-20 lines of surrounding context at each thread location and return a per-thread summary.

   Then spawn a `code-writer` subagent (model: sonnet) with:
   - Per-thread context: file path, line number(s), full comment bodies from all comments in each thread (later replies often contain clarifications)
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md, passed inline in the prompt as constraints
   - Reference to the project's CLAUDE.md for project-specific build/test commands
   - For bot threads: classify each as **Applicable** (the bot's finding is valid — fix the code) or **Not applicable** (false positive or red herring — the flagged pattern is intentional, the suggestion doesn't fit this context, or the issue was already addressed — do NOT fix the code)
   - Group threads by file path to minimize context switching; one subagent handles all threads

   **Anti-inline rules:**
   - The orchestrator MUST NOT read source files to analyze or diagnose findings — pass thread details (file, line, comment body) to the code-writer and let it investigate
   - MUST delegate even for a single thread — the bulk threshold only gates the Explore subagent, not delegation itself

5. **Scope-check fixes**: The code-writer spawned in step 4 runs its own verification internally — re-running here would duplicate work. Instead, run `git diff --name-only` and compare against the file paths from the thread list. Flag any unexpected files (not referenced by any thread) to the user via AskUserQuestion before proceeding. If any fix is incomplete after 2 attempts, mark it as unresolved and continue with remaining threads.

6. **React to not-applicable bot threads**: For each bot thread classified as not applicable in step 4:
   - Add a thumbs-down reaction to the first comment: `gh api repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions -f content="-1"`
   - Draft a brief reply explaining why the finding doesn't apply (e.g., "False positive -- this pattern is intentional because ..." or "Not applicable -- the flagged code path is unreachable from ...")
   - Post the reply directly (bots are autonomous per step 3) -- write to temp file, sanitize, and post per references/github-text.md

   The `{owner}/{repo}` and `{comment_id}` come from the PR comments data returned by get-pr-comments.sh. Use `gh api graphql` to resolve the REST comment ID from a node ID if needed.

7. **Commit and push**: Stage changed files. Commit with message "Address PR review feedback: <brief summary of threads fixed>" per the Inline Commit Procedure in references/commit-message-format.md, then push with `git push`. Record the commit hash with `git rev-parse HEAD` — it is needed for reply links in step 9.

8. **Handle description threads**: If description threads were classified in step 3, route them to the Description Path starting at step 1 (Ensure branch context). Pass the thread comments as quality findings in step 2. Do NOT send description threads to the code-writer. Description threads produce no code commit, so replies in step 9 reference the PR URL (not a commit hash). Skip this step if no description threads exist.

9. **Draft replies**: After code fixes are committed, draft a reply for each code thread and each description thread:
   - **Fixed** (code was changed): add a thumbs-up reaction (`gh api repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions -f content="+1"`) and reply "Fixed in <commit-url>" using the commit hash from step 7. For bot threads, post directly. Human thread replies go through approval in step 10.
   - **Description refreshed** (description thread handled in step 8): add a thumbs-up reaction and reply with the PR URL (no commit hash — description refreshes don't produce code commits). Bot/human approval rules still apply.
   - **Already addressed** (no code change needed): reply concisely that it's already addressed.
   - **Needs discussion**: draft a thoughtful response.

10. **Present drafts for approval**: Show each draft alongside the reviewer's comment for context. For each human thread draft, present options via AskUserQuestion: "Approve", "Skip", "Edit". MUST NOT post any reply to a human reviewer's comment without showing the draft and receiving explicit user approval.

11. **Post approved replies and bot replies**: For each reply, write the text to `./tmp/reply.txt` using Bash (`mkdir -p ./tmp && cat <<'EOF' > ./tmp/reply.txt` ... `EOF`), sanitize in place with `~/.claude/skills/git/scripts/sanitize.sh ./tmp/reply.txt`, then post using the in-thread reply endpoint:

    ```bash
    gh api repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies -F body=@./tmp/reply.txt
    ```

    Where `{comment_id}` is the REST ID of the **first comment in the thread** being replied to (available from the get-pr-comments.sh output), and `{owner}`, `{repo}`, `{pull_number}` also come from that script's output.

    MUST NOT create new standalone review comments by passing `commit_id`, `path`, `line`, or `side` parameters -- that opens a new thread instead of replying in the existing one. MUST NOT use `gh pr comment` for review thread replies -- that posts a PR-level comment, not an in-thread reply.

    Clean up temp files after all replies are posted.

12. **Report**: Confirm which threads were fixed, which replies were posted, which were skipped, and which remain unresolved.

---

### Combined Path

When both CI failures and unresolved review threads exist:

1. Run the **CI path** first (through commit and push).
2. Run the **Review path** (all steps including replies). This produces a second push -- that is expected.
3. Report a unified summary: CI fixes applied, review fixes applied, replies posted, and any unresolved items.

---

### Description Path

Improve a PR description that is a wall of text, missing diff coverage, or lacks verification info. Also invoked by Review Path step 8 when review threads flag description inaccuracy.

1. **Ensure branch context**: Check if the branch context file exists (path per references/git-patterns.md "Branch Context File").
   - If **missing**: run the Branch Context Creation pattern from `references/git-patterns.md`.
   - If the file contains the `N/A` sentinel: ask via AskUserQuestion -- "The PR description could be improved, but there's no branch context. What's the motivation for this branch?" with options: **"I'll explain"** (user provides the reason; write it to the branch context file) or **"Skip description update"** (stop this path entirely).
   - If the file has real content but is a single sentence AND the diff spans 20+ files or 3+ top-level directories: run the context adequacy check from push.md's "Context adequacy check" step (ask user if they want to update context before proceeding).

2. **Present findings**: Show the user which quality issues were detected (wall of text, missing coverage, missing verification info) and ask via AskUserQuestion: "Refresh the PR description?" with options: **"Refresh it"** (proceed to step 3) or **"Skip"** (stop this path).

3. **Refresh**: Run the Refresh Description mode from operations/push.md starting at step 4 (pr-writer delegation) -- steps 1-3 (PR check, branch context, adequacy) are already covered by Detection step 3 and Description Path step 1 above. Pass the quality findings as the `context` field (e.g., "Description was flagged as a wall of text for a multi-concern PR -- restructure with numbered list").

4. **Report**: Confirm the description was updated and show the PR URL.
