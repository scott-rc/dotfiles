# Fix

Auto-detect and fix CI failures, unresolved review threads, and PR description quality issues.

## Instructions

### Stack Mode

When the user asks to fix "the stack" / "all PRs" / "every branch," run Fix across every branch in the stack that has a PR.

1. **Enumerate branches**: Run `git-spice log short --json | jq -r 'select(.change != null) | .name'` to get all branches with PRs. If none have PRs, report and stop.
2. **Record origin**: Save the current branch name to restore later.
3. **Iterate bottom-to-top**: For each branch (in the order returned, which is top-to-bottom — reverse it):
   - `git-spice branch checkout <name> --no-prompt`
   - Run the standard Detection and path flow below (Detection through each path's final step)
   - In the "Neither" routing, skip the cron/loop check — just report the branch as green and continue
   - Report per-branch results before moving to the next branch
4. **Restore**: Checkout the original branch.
5. **Unified report**: Summarize per-branch results (what was fixed, what was already green). If ALL branches are green, check `CronList` — if a `/git fix` loop is active and all CI checks across every branch have reached terminal state, suggest canceling it.

When NOT in stack mode (single-branch fix), proceed to Detection below.

---

### Detection

Run these three checks in parallel:

User-provided instructions (e.g., "fix and ensure contributing is up to date") supplement but do not replace Detection. MUST run all three checks regardless of additional context. Handle side-requests as a separate concern after the standard fix paths complete.

1. **Review threads**: Run `~/.claude/skills/git/scripts/get-pr-comments.sh --count` to get the unresolved thread count.
2. **CI status**: Follow the "CI Detection" pattern in references/git-patterns.md (steps 1 and 2). Group checks by status (failed, pending, passed).
3. **PR description quality**: Fetch the current PR body (`gh pr view --json body,number -q '.body'`). If no PR exists, skip. Detect base branch per references/git-patterns.md. Run `git diff --stat origin/<base>...HEAD` and assess:

   The description needs a refresh if ANY of:
   - **Missing diff coverage**: The `--stat` output shows file groups (e.g., `.claude/`, `.github/`, `docs/`, test files) that the body doesn't mention at all -- not even in passing.
   - **Missing verification info**: The diff includes CI workflow files (`.github/workflows/`, `.buildkite/`), deploy configs, or build system changes, but the body says nothing about how to verify locally or what CI triggers.

   If neither triggers, mark the description as OK. Note: review threads that flag description inaccuracy are handled by Review Path step 3 classification, not here.

After all three complete, route based on results:

- **CI failures only** (no unresolved threads, description OK): follow the CI path below.
- **Unresolved threads only** (CI green or no CI, description OK): follow the Review path below.
- **Both CI failures and unresolved threads** (description OK): follow the Combined path below.
- **Description issues only** (CI green, no unresolved threads): follow the Description path below.
- **Neither** (CI green, no unresolved threads, description OK): report that everything is green. Then run `CronList` — if any job's prompt contains `/git fix`, check whether all CI checks have reached a terminal state (passed or failed). If any checks are still pending, in progress, or in a non-terminal state (including statuses like "OTHER"), keep the loop running and tell the user (e.g., "All passing so far, but N checks still in progress — keeping the loop active"). Only cancel with `CronDelete` when every check has reached a terminal state and all threads are resolved.

If CI, Review, or Combined path ran AND the description quality check (missing coverage or missing verification info) also flagged issues, run the Description path after all other paths complete. Description threads found by Review Path step 3 are handled by Review Path step 8 — they do not trigger this rule.

---

### CI Path

1. **Detect CI system** per the "CI System Detection" section in references/git-patterns.md: check `.github/workflows/` for github-actions, `.buildkite/` for buildkite.

   - **GitHub Actions**: Use `~/.claude/skills/git/scripts/get-failed-runs.sh` to get failed run IDs and workflow names. If the script returns an empty array but detection found failures, runs may still be initializing -- report to the user and suggest retrying shortly. Detect base branch per references/git-patterns.md. Classify each failed run inline using references/ci-triage.md: fetch logs, apply transient/flake/real criteria, and rerun if appropriate.

     Based on the classification:
     - **transient** or **flake**: report the classification to the user (rerun already triggered). Stop.
     - **real**: proceed to the fix step using the trimmed logs and root cause from the triage.

   - **Buildkite**: Fetch logs per references/buildkite-handling.md. Skip automated triage for Buildkite -- treat all failures as real. Truncate logs to the last 200 lines per job. **Before fixing**, present a summary of failed jobs (job name + one-line failure snippet from each log) and ask the user with options: "Fix all", "Skip (likely flakes)", "Let me choose". If "Let me choose", present each job individually and let the user select which to fix. Only fix jobs the user selected.

2. **Fix the issues directly**, using:
   - For GitHub Actions: the triage results as context (root cause analysis, trimmed failure logs, relevant file paths)
   - For Buildkite: the truncated failure logs
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md
   - The project's CLAUDE.md for project-specific build/test commands

   If the fix is ambiguous or risky, present candidate fixes as options before applying. If the failure is in CI configuration (not source code), explain what needs to change and confirm with the user before applying.

3. **Commit and push**: Summarize what failed, why, what was fixed, and whether local verification passed. Stage changed files, commit with message "Fix <workflow/check name> CI failure: <brief cause>" per the Inline Commit Procedure in references/commit-message-format.md. Then check PR existence via the Stack Metadata via JSON pattern in references/git-spice-patterns.md (`.change` field): if a PR exists, push with `git-spice branch submit --update-only --no-prompt`; otherwise use `git-spice branch submit --no-publish --no-prompt`.

---

### Review Path

1. **Check for PR**: If Detection step 1 reported no PR, stop.

2. **Present a summary**: Run `get-pr-comments.sh --summary` and present its compact output. For large thread sets, group by file and show counts rather than listing every thread individually.

3. **Classify threads**: Classify each thread by commenter type — **bot threads** (bugbot, dependabot, or any automated bot) are handled autonomously; **human reviewer threads** require explicit user approval before applying code fixes. Agents MUST NOT post replies to human reviewer threads. MUST ask the user to confirm before applying code fixes to human threads.

   Independently classify each thread by **target**: **code threads** (feedback about source code, tests, configs, or CI files) vs **description threads** (feedback about the PR title, summary, or description text being inaccurate, misleading, or incomplete — the comment discusses what the PR *says*, not what it *does*). Classify each thread on both axes independently — a thread may be both a bot thread and a description thread.

4. **Gather context and fix**: This step applies to **code threads** only. Skip if all threads are description threads.

   Check the bulk threshold (>=5 threads OR <5 touching >3 files). If at or above the threshold: spawn an Explore subagent (model: sonnet) to read each referenced file with 10-20 lines of surrounding context at each thread location and return a per-thread summary.

   Then fix the issues directly, using:
   - Per-thread context: file path, line number(s), full comment bodies from all comments in each thread (later replies often contain clarifications)
   - The local fix commands resolved from "Local Fix Commands" in references/git-patterns.md
   - The project's CLAUDE.md for project-specific build/test commands
   - For bot threads: classify each as **Applicable** (the bot's finding is valid — fix the code) or **Not applicable** (false positive or red herring — the flagged pattern is intentional, the suggestion doesn't fit this context, or the issue was already addressed — do NOT fix the code)
   - Self-classify each applicable finding's fix approach: if it's a security or correctness bug (path traversal, injection, data loss, missing validation with observable wrong behavior), write a failing test demonstrating the bug first, then fix. If it's consistency or style (pattern matching, naming, formatting), fix and verify existing tests pass — no new test required. Tiebreaker: if the missing handling could cause data loss or silent wrong results, treat as test-first.
   - Group threads by file path to minimize context switching

5. **Scope-check fixes**: After fixing, run `git diff --name-only` and compare against the file paths from the thread list. Flag any unexpected files (not referenced by any thread) to the user before proceeding.

6. **React to not-applicable bot threads**: For each bot thread classified as not applicable in step 4:
   - Add a thumbs-down reaction to the first comment: `gh api repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions -f content="-1"`
   - Draft a brief reply explaining why the finding doesn't apply (e.g., "False positive -- this pattern is intentional because ..." or "Not applicable -- the flagged code path is unreachable from ...")
   - Post the reply directly (bots are autonomous per step 3) -- write to temp file, sanitize, and post per references/github-text.md

   The `{owner}/{repo}` and `{comment_id}` come from the PR comments data returned by get-pr-comments.sh. Use `gh api graphql` to resolve the REST comment ID from a node ID if needed.

7. **Commit and push**: Stage changed files. Commit with message "Address PR review feedback: <brief summary of threads fixed>" per the Inline Commit Procedure in references/commit-message-format.md. Then check PR existence via the Stack Metadata via JSON pattern in references/git-spice-patterns.md (`.change` field): if a PR exists, push with `git-spice branch submit --update-only --no-prompt`; otherwise use `git-spice branch submit --no-publish --no-prompt`. Record the commit hash with `git rev-parse HEAD` — it is needed for reply links in step 9.

8. **Handle description threads**: If description threads were classified in step 3, route them to the Description Path starting at step 1 (Ensure branch context). Pass the thread comments as quality findings in step 2. Do NOT apply code fixes for description threads. Description threads produce no code commit, so replies in step 9 reference the PR URL (not a commit hash). Skip this step if no description threads exist.

9. **Draft replies**: After code fixes are committed, draft a reply for each code thread and each description thread:
   - **Fixed** (code was changed): add a thumbs-up reaction (`gh api repos/{owner}/{repo}/pulls/comments/{comment_id}/reactions -f content="+1"`) and reply "Fixed in <commit-url>" using the commit hash from step 7. For bot threads, post directly. Human thread replies go through approval in step 10.
   - **Description refreshed** (description thread handled in step 8): add a thumbs-up reaction and reply with the PR URL (no commit hash — description refreshes don't produce code commits). Bot/human approval rules still apply.
   - **Already addressed** (no code change needed): reply concisely that it's already addressed.
   - **Needs discussion**: draft a thoughtful response.

10. **Present drafts for approval**: Show each draft alongside the reviewer's comment for context. For each human thread draft, present options: "Approve", "Skip", "Edit". MUST NOT post any reply to a human reviewer's comment without showing the draft and receiving explicit user approval.

11. **Post approved replies and bot replies**: For each reply, write the text to `./tmp/pr/<pr_number>/reply-<comment_id>.txt` using Bash (`mkdir -p ./tmp/pr/<pr_number> && cat <<'EOF' > ./tmp/pr/<pr_number>/reply-<comment_id>.txt` ... `EOF`), sanitize in place with `~/.claude/skills/git/scripts/sanitize.sh ./tmp/pr/<pr_number>/reply-<comment_id>.txt`, then post using the in-thread reply endpoint:

    ```bash
    gh api repos/{owner}/{repo}/pulls/{pull_number}/comments/{comment_id}/replies \
      -F body=@./tmp/pr/<pr_number>/reply-<comment_id>.txt
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

Improve a PR description that is missing diff coverage or lacks verification info. Also invoked by Review Path step 8 when review threads flag description inaccuracy.

1. **Ensure branch context**: Check if the branch context file exists (path per references/git-patterns.md "Branch Context File").
   - If **missing**: run the Branch Context Creation pattern from `references/git-patterns.md`.
   - If the file contains the `N/A` sentinel: ask the user -- "The PR description could be improved, but there's no branch context. What's the motivation for this branch?" with options: **"I'll explain"** (user provides the reason; write it to the branch context file) or **"Skip description update"** (stop this path entirely).
   - If the file has real content but is a single sentence AND the diff spans 20+ files or 3+ top-level directories: run the context adequacy check from operations/push.md's "Context adequacy check" step (ask user if they want to update context before proceeding).

2. **Present findings**: Show the user which quality issues were detected (missing coverage, missing verification info) and ask: "Refresh the PR description?" with options: **"Refresh it"** (proceed to step 3) or **"Skip"** (stop this path).

3. **Refresh**: Run the Refresh Description mode from operations/push.md starting at the "Write PR title and description" step -- the PR check, branch context, and adequacy steps are already covered by Detection step 3 and Description Path step 1 above. Pass the quality findings as the `context` field (e.g., "Description was missing coverage for CI workflow changes").

4. **Report**: Confirm the description was updated and show the PR URL.
