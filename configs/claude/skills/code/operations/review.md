# Review

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

1. **Identify review scope**:
   Determine what code to review. If the user specifies files, functions, or a diff, use that. If unspecified, ask what they want reviewed.

2. **Assess scope size**:
   Count changed files and lines (`git diff --stat` or file list).
   - **Quick** (≤8 files AND ≤500 lines, user didn't request thorough) → steps 3–6.
   - **Auto-thorough** (any of: user said "thorough"/"deep review", >20 files, or >1500 lines) → steps 3, then 7–11. No confirmation needed.
   - **Ask** (between the two thresholds) → present an AskUserQuestion: "Thorough review (subagent decomposition)" or "Quick review (single-pass)", then route accordingly.

3. **Load guidelines**: Read references/load-guidelines.md for the full list of guidelines. Load references/general-guidelines.md and any applicable language-specific files. Skip references/testing-guidelines.md only if the scope contains no test files.

---

### Quick Path (steps 4–6)

4. **Spawn review subagent**:
   Spawn a Task subagent (type: code-reviewer). Pass it:
   - The files to review
   - references/review-checklist.md for review criteria
   - Paths to the guideline files loaded in step 3
   - Project context (repo root, conventions observed)

5. **Report findings**:
   MUST present findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
   - File and location
   - What the problem is (one sentence)
   - A concrete fix or recommendation

   If no findings, say so — do not manufacture issues.

6. **Stop** — quick review is complete.

---

### Thorough Path (steps 7–11)

7. **Decompose into review scopes**:
   Spawn a Task subagent (type: Explore) to analyze the diff and propose review scopes. The subagent MUST:
   - Run `git diff --stat` (or inspect the file list) to map all changed files and line counts
   - Group related files by theme/concern (e.g., "API handler changes", "test updates", "config/infra")
   - Propose 2–5 non-overlapping scopes, each with a name, concrete file list, focus, and scope-specific criteria beyond the standard checklist
   - Return the scopes as a structured list

   Present the proposed scopes to the user via AskUserQuestion for confirmation or adjustment before proceeding.

8. **Spawn review subagents**:
   Spawn parallel Task subagents (type: code-reviewer), one per scope. Pass each subagent:
   - The scope's name, file list, focus, and scope-specific criteria
   - references/review-checklist.md for review criteria
   - Paths to the guideline files loaded in step 3
   - Project context (repo root, conventions observed)

9. **Consolidate findings**:
   Merge all subagent findings. Deduplicate by file + location. Group by severity (issues → suggestions → nits).

10. **Report findings**:
    Same format as step 5 — grouped by severity, each with file/location, problem description, and concrete fix.

11. **Offer fix plan**:
    If any issues or suggestions were found, ask the user if they want a fix plan. If yes, invoke the compose skill: `skill: "compose", args: "plan fixes from the review findings"`.
