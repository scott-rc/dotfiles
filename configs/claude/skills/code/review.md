# Review Operation

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

1. **Identify review scope**:
   Determine what code to review. If the user specifies files, functions, or a diff, use that. If unspecified, ask what they want reviewed.

2. **Assess scope size**:
   Count changed files and lines (`git diff --stat` or file list).
   - If ≤8 changed files AND ≤500 changed lines, continue to step 3.
   - If >8 changed files OR >500 changed lines, present an AskUserQuestion with options: "Thorough review (subagent decomposition)", "Quick review (single-pass)". If the user chooses quick review, continue to step 3. If thorough (or the user said "thorough review" or "deep review"), skip to step 7.

3. **Load guidelines**: Read [load-guidelines.md](load-guidelines.md) for the full list of guidelines. Load general-guidelines.md and any applicable language-specific files. Skip testing-guidelines.md only if the scope contains no test files.

4. **Spawn review subagent**:
   Read [review-template.md](review-template.md) for the subagent prompt template. Spawn a Task subagent (type: code-reviewer) with all files in the scope. Fill in the template with:
   - All files in the review scope
   - Focus: general review
   - Paths to the guidelines files loaded in step 3
   - The checklist from [review-checklist.md](review-checklist.md) — copy it into the template's checklist placeholder

5. **Report findings**:
   MUST present findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
   - File and location
   - What the problem is (one sentence)
   - A concrete fix or recommendation

   If no findings, say so — do not manufacture issues.

6. **Stop** — single-pass review is complete. Skip remaining steps.

7. **Load guidelines** (large scope): Read [load-guidelines.md](load-guidelines.md). Load general-guidelines.md and any applicable language-specific files.

8. **Decompose into review scopes**:
   Spawn a Task subagent (type: Explore) to analyze the diff and propose review scopes. The subagent MUST:
   - Run `git diff --stat` (or inspect the file list) to map all changed files and line counts
   - Group related files by theme/concern (e.g., "API handler changes", "test updates", "config/infra")
   - Propose 2–5 non-overlapping scopes, each with a name, concrete file list, focus, and scope-specific criteria beyond the standard checklist
   - Return the scopes as a structured list

   Present the proposed scopes to the user via AskUserQuestion for confirmation or adjustment before proceeding.

9. **Load review template**: Read [review-template.md](review-template.md) for the subagent prompt template.

10. **Spawn review subagents**:
    Spawn parallel Task subagents (type: code-reviewer), one per scope. Fill in the template with:
    - The scope's name, file list, and focus
    - Paths to the guidelines files loaded in step 7
    - Project context (repo root, conventions observed)
    - The checklist from [review-checklist.md](review-checklist.md) — copy it into the template's checklist placeholder
    - Scope-specific criteria

11. **Consolidate findings**:
    Merge all subagent findings. Deduplicate by file + location. Group by severity (issues → suggestions → nits).

12. **Report findings**:
    Same format as step 5 — grouped by severity, each with file/location, problem description, and concrete fix.

13. **Offer fix plan**:
    If any issues or suggestions were found, ask the user if they want a fix plan. If yes, invoke the compose skill: `skill: "compose", args: "plan fixes from the review findings"`.
