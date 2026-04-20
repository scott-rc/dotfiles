# Review Rules

Evaluate a CLAUDE.md or scoped rules file against best practices, report findings grouped by severity, and fix issues via the review-fix loop until all agents pass.

## Instructions

1. **Locate the rules file(s)**:
   - If the user provides a path, use it directly
   - If the user says "review my CLAUDE.md", check the current project root first, then `~/.claude/CLAUDE.md`
   - If neither, discover CLAUDE.md files in the project and present them as options
   - SHOULD also identify related files: other CLAUDE.md files in parent/child directories, `.claude/rules/` files (including subdirectories), `~/.claude/rules/` user-level rules

2. **Evaluate rules via multi-perspective review**:
   Spawn 2 subagents in parallel per references/multi-perspective-review.md, substituting `<spec-file>` = `configs/claude/skills/compose/references/rules-spec.md` and `<target>` = the rules file (plus related files listed in step 1).

   - **Sonnet** — checklist compliance: structure, `@file` usage, anti-patterns from references/rules-spec.md
   - **Opus** — internal consistency: contradictions between files in the hierarchy, missing guidance, edge cases

   Each agent reads the target file and all related files (`@file` references, other CLAUDE.md files in the hierarchy, `.claude/rules/` files).

3. **Synthesize findings**:
   Merge results from both agents into a single list grouped by severity (Blocking > Improvement > Suggestion). Deduplicate overlapping findings. Cross-reference against project-specific context the agents would not have (e.g., known issues where Claude ignores specific rules, recently changed conventions). See references/multi-perspective-review.md for disagreement handling.

4. **Present findings**:
   Group results by severity (Blocking, Improvement, Suggestion) per references/quality-checklist.md. For each finding, state: what the issue is, which file and line/section it's in, what the fix would be.

5. **Review-fix loop**:
   Run the loop per references/multi-perspective-review.md. Apply fixes inline using Edit/Write. Findings follow `file:line — severity — one-sentence problem` format. Iterate until the loop terminates per the Termination section in multi-perspective-review.md.

6. **Report outcomes**:
   Present a summary of what was reviewed, what was fixed, and what remains (pass/fail, cycle count, unresolved findings with severity and reason).
