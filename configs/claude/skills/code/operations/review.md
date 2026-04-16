# Review

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

**Loop mode**: When the user requests "loop", "review and loop", or "review and fix loop" — or when invoked programmatically by another operation — the review operation drives an evaluate-fix cycle after the initial evaluation. Both quick and thorough paths feed into the loop phase (steps 15–20) instead of stopping or offering a fix plan. Without loop mode, the operation behaves as a single-pass review. In loop mode, skip all user-facing prompts (scope-size confirmation in step 3, scope decomposition confirmation in step 9) — default to thorough for any scope exceeding the quick threshold.

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
   - **Ask** (between the two thresholds) → ask the user: "Thorough review (subagent decomposition)" or "Quick review (single-pass)", then route accordingly.

4. **Load guidelines**: Read references/load-guidelines.md for the full list of guidelines. Load references/general-guidelines.md and any applicable language-specific files. Skip references/testing-guidelines.md only if the scope contains no test files.

---

### Quick Path (steps 5–8)

5. **Review the code inline**:
   Review the code inline, using:
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
   Write `./tmp/branches/<sanitized-branch>/review.md` (create the directory if needed) using the review artifact format below.

8. **Stop or loop**:
   If NOT in loop mode — quick review is complete. Stop.
   If in loop mode — proceed to the Loop Phase (steps 15–20).

---

### Thorough Path (steps 9–14)

9. **Decompose into review scopes**:
   Spawn a Task subagent (type: Explore) to analyze the diff and propose review scopes. The subagent MUST:
   - Run `git diff --stat` (or inspect the file list) to map all changed files and line counts
   - Group related files by theme/concern (e.g., "API handler changes", "test updates", "config/infra")
   - Propose 2–5 non-overlapping scopes, each with a name, concrete file list, focus, and scope-specific criteria beyond the standard checklist
   - Return the scopes as a structured list

   Present the proposed scopes to the user for confirmation or adjustment before proceeding. (Skipped in loop mode — use proposed scopes directly.)

10. **Spawn review subagents**:
    Spawn parallel subagents, one per scope. Pass each:
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
    Write `./tmp/branches/<sanitized-branch>/review.md` (create the directory if needed) using the review artifact format below.

14. **Offer fix plan or loop**:
    If NOT in loop mode — if any issues or suggestions were found, ask the user if they want a fix plan. If yes, invoke the compose skill: `skill: "compose", args: "plan fixes from the review findings"`.
    If in loop mode — proceed to the Loop Phase (steps 15–20).

---

### Loop Phase (steps 15–20)

15. **Check for actionable findings**:
    If no Blocking or Improvement findings exist after the initial evaluation, the review converged on the first pass — skip to step 20.

16. **Fix findings**:
    Fix all Blocking and Improvement findings directly. For style, convention, and structural fixes, apply and verify existing tests pass. For behavioral correctness issues (e.g., missing error check, wrong return value), write a regression test first. No pause, no user confirmation. Use:
    - The findings grouped by file, with file paths and line numbers
    - The guideline files loaded in step 4
    - Project context (repo root, conventions observed)

    Handle Suggestions per the project's loop rules: fix if quick (fewer than 3 per file); otherwise note and move on.

17. **Re-evaluate**:
    Re-review only the files that were fixed in step 16 — not the full original scope. Pass the same guidelines and checklist from step 4, plus the list of findings delegated in step 16 with the instruction "verify these specific issues were addressed."

18. **Check convergence**:
    If no Blocking or Improvement findings remain, proceed to step 20. If findings remain and iteration count < max iterations (default: 4), return to step 16. If a recurring finding persists after a fix attempt, escalate to the user or record as "acknowledged, not addressed" with rationale, per the project's loop rules.

19. **Report loop status**:
    If max iterations reached without convergence, present remaining findings with their status and let the user decide.

20. **Persist final findings**:
    Write `./tmp/branches/<sanitized-branch>/review.md` using the review artifact format below. Add loop metadata fields to the Metadata section:
    - iterations_completed: <n>
    - convergence_status: converged | max_iterations_reached | escalated

---

### Review Artifact Format

Used by steps 7, 13, and 20. Create the directory if needed.

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
