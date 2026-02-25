# Review Code

Evaluate code for test gaps, idiomaticity, simplification opportunities, and other issues — producing a structured findings report.

## Instructions

1. **Identify review scope**:
   Determine what code to review. If the user specifies files, functions, or a diff, use that. If unspecified, ask what they want reviewed.

2. **Assess scope size**:
   Count changed files and lines (`git diff --stat` or file list). Apply this heuristic:
   - **Small scope**: ≤8 changed files AND ≤500 changed lines → follow steps 3–6
   - **Large scope**: >8 changed files OR >500 changed lines → follow steps 7–13

   For small scope, proceed directly to step 3. For large scope, present an AskUserQuestion with options: "Thorough review (subagent decomposition)", "Quick review (single-pass)". The user can also force the thorough path by saying "thorough review" or "deep review".

### Small Scope (Single-Pass Review)

3. **Load guidelines**: MUST load [general-guidelines.md](general-guidelines.md) and [testing-guidelines.md](testing-guidelines.md). If a language-specific file exists for the target language ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [rust-guidelines.md](rust-guidelines.md), [shell-guidelines.md](shell-guidelines.md)), load it too.

4. **Load review template**: Read [review-template.md](review-template.md) for the subagent prompt template.

5. **Spawn review subagent**:
   Launch a single Task subagent (type: code-reviewer) with all files in the scope. Fill in the template with:
   - All files in the review scope
   - Focus: general review
   - Paths to the guidelines files loaded in step 3
   - The checklist from [review-checklist.md](review-checklist.md) — copy it into the template's checklist placeholder

6. **Report findings**:
   MUST present findings grouped by severity (issues first, then suggestions, then nits). Each finding MUST include:
   - File and location
   - What the problem is (one sentence)
   - A concrete fix or recommendation

   If no findings, say so — do not manufacture issues.

### Large Scope (Subagent Decomposition)

7. **Load guidelines**: MUST load [general-guidelines.md](general-guidelines.md). If language-specific files exist for the target languages ([typescript-guidelines.md](typescript-guidelines.md), [go-guidelines.md](go-guidelines.md), [rust-guidelines.md](rust-guidelines.md), [shell-guidelines.md](shell-guidelines.md)), load them too.

8. **Decompose into review scopes**:
   Spawn a Task subagent (type: Explore) to analyze the diff and propose review scopes. The subagent MUST:
   - Run `git diff --stat` (or inspect the file list) to map all changed files and line counts
   - Group related files by theme/concern (e.g., "API handler changes", "test updates", "config/infra")
   - Propose 2–5 non-overlapping review scopes, each with:
     - **Name** — short descriptive label
     - **Files** — concrete file list (every changed file assigned to exactly one scope)
     - **Focus** — what to pay attention to in this scope
     - **Criteria** — scope-specific review criteria beyond the standard checklist
   - Return the scopes as a structured list (not raw diff output)

   Present the proposed scopes to the user via AskUserQuestion for confirmation or adjustment before proceeding.

9. **Load review template**: Read [review-template.md](review-template.md) for the subagent prompt template. Use this template when spawning each review subagent below.

10. **Spawn review subagents**:
    Launch parallel Task subagents (type: code-reviewer), one per scope. Fill in the template with:
    - The scope's name, file list, and focus
    - Paths to the guidelines files loaded in step 7
    - Project context (repo root, conventions observed)
    - The checklist from [review-checklist.md](review-checklist.md) — copy it into the template's checklist placeholder
    - Scope-specific criteria

11. **Consolidate findings**:
    Merge all subagent findings. Deduplicate by file + location. Group by severity (issues → suggestions → nits).

12. **Report findings**:
    Same format as step 6 — grouped by severity, each with file/location, problem description, and concrete fix.

13. **Offer fix plan**:
    If any issues or suggestions were found, ask the user if they want a fix plan. If yes, delegate to compose: `skill: "compose", args: "plan fixes from the review findings"`.
