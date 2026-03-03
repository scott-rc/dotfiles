# Review

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

1. **Identify review scope**:
   Determine what code to review. If the user specifies files, functions, or a diff, use that. If unspecified, ask what they want reviewed.

2. **Check for prior findings**:
   Get the current branch name (`git rev-parse --abbrev-ref HEAD`) and sanitize it by replacing `/` with `--`. Resolve the prior findings path as `./tmp/branches/<sanitized-branch>/review.md`. If the file exists, read it and compare its `head_sha` against `git rev-parse HEAD`:
   - If `head_sha` matches current HEAD — the review is fully current; load findings but note no new commits exist.
   - If `head_sha` differs — new commits exist since the last review; load findings and note that new commits have been pushed (prior findings were likely addressed or new work added).
   - If the file does not exist — first review; no prior context.

3. **Assess scope size**:
   Count changed files and lines (`git diff --stat` or file list).
   - **Quick** (≤8 files AND ≤500 lines, user didn't request thorough) → steps 5–8.
   - **Auto-thorough** (any of: user said "thorough"/"deep review", >20 files, or >1500 lines) → steps 5, then 9–14. No confirmation needed.
   - **Ask** (between the two thresholds) → present an AskUserQuestion: "Thorough review (subagent decomposition)" or "Quick review (single-pass)", then route accordingly.

4. **Load guidelines**: Read references/load-guidelines.md for the full list of guidelines. Load references/general-guidelines.md and any applicable language-specific files. Skip references/testing-guidelines.md only if the scope contains no test files.

---

### Quick Path (steps 5–8)

5. **Spawn review subagent**:
   Spawn a Task subagent (type: code-reviewer). Pass it:
   - The files to review
   - references/review-checklist.md for review criteria
   - Paths to the guideline files loaded in step 4
   - Project context (repo root, conventions observed)
   - If prior findings were loaded in step 2: the full prior findings list with the instruction "These findings were identified in a prior review of this branch. [If head_sha matched: they are fully current — do not re-flag unless a fix is demonstrably incorrect or introduced a new issue.] [If head_sha differed: new commits have been pushed since this review — the findings were likely addressed or superseded. Pass them as historical context only; focus on finding NEW issues.]"

6. **Report findings**:
   MUST present findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
   - File and location
   - What the problem is (one sentence)
   - A concrete fix or recommendation

   If no findings, say so — do not manufacture issues.

7. **Persist findings**:
   Write `./tmp/branches/<sanitized-branch>/review.md` (create the directory if needed) with this format:

   ```markdown
   # Code Review: <branch>

   ## Metadata
   - head_sha: <git rev-parse HEAD>
   - base: <base branch>
   - reviewed_at: <ISO 8601 timestamp>

   ## Findings
   <full findings list in same format as report>

   ## User Decisions
   <any skipped findings or chosen approaches>
   ```

8. **Stop** — quick review is complete.

---

### Thorough Path (steps 9–14)

9. **Decompose into review scopes**:
   Spawn a Task subagent (type: Explore) to analyze the diff and propose review scopes. The subagent MUST:
   - Run `git diff --stat` (or inspect the file list) to map all changed files and line counts
   - Group related files by theme/concern (e.g., "API handler changes", "test updates", "config/infra")
   - Propose 2–5 non-overlapping scopes, each with a name, concrete file list, focus, and scope-specific criteria beyond the standard checklist
   - Return the scopes as a structured list

   Present the proposed scopes to the user via AskUserQuestion for confirmation or adjustment before proceeding.

10. **Spawn review subagents**:
    Spawn parallel Task subagents (type: code-reviewer), one per scope. Pass each subagent:
    - The scope's name, file list, focus, and scope-specific criteria
    - references/review-checklist.md for review criteria
    - Paths to the guideline files loaded in step 4
    - Project context (repo root, conventions observed)
    - If prior findings were loaded in step 2: the full prior findings list with the instruction "These findings were identified in a prior review of this branch. [If head_sha matched: they are fully current — do not re-flag unless a fix is demonstrably incorrect or introduced a new issue.] [If head_sha differed: new commits have been pushed since this review — the findings were likely addressed or superseded. Pass them as historical context only; focus on finding NEW issues.]"

11. **Consolidate findings**:
    Merge all subagent findings. Deduplicate by file + location. Group by severity (issues → suggestions → nits).

12. **Report findings**:
    Same format as step 6 — grouped by severity, each with file/location, problem description, and concrete fix.

13. **Persist findings**:
    Write `./tmp/branches/<sanitized-branch>/review.md` (create the directory if needed) with this format:

    ```markdown
    # Code Review: <branch>

    ## Metadata
    - head_sha: <git rev-parse HEAD>
    - base: <base branch>
    - reviewed_at: <ISO 8601 timestamp>

    ## Findings
    <full findings list in same format as report>

    ## User Decisions
    <any skipped findings or chosen approaches>
    ```

14. **Offer fix plan**:
    If any issues or suggestions were found, ask the user if they want a fix plan. If yes, invoke the compose skill: `skill: "compose", args: "plan fixes from the review findings"`.
