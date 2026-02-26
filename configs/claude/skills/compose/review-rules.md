# Review Rules

Evaluate a CLAUDE.md or scoped rules file against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the rules file(s)**:
   - If the user provides a path, use it directly
   - If the user says "review my CLAUDE.md", check the current project root first, then `~/.claude/CLAUDE.md`
   - If neither, discover CLAUDE.md files in the project and present them as AskUserQuestion options
   - SHOULD also identify related files: other CLAUDE.md files in parent/child directories, `.claude/rules/` files (including subdirectories), `~/.claude/rules/` user-level rules

2. **Evaluate rules via multi-perspective review**:
   Spawn 3 Task subagents in parallel per [multi-perspective-review.md](multi-perspective-review.md), all type: `rules-reviewer`:

   - **Sonnet** — checklist compliance: structure, `@file` usage, anti-patterns from [rules-spec.md](rules-spec.md)
   - **Opus** — internal consistency: contradictions between files in the hierarchy, missing guidance, edge cases
   - **Haiku** — token efficiency: common knowledge, duplicated content, over-specification

   Each agent reads the target file and all related files (`@file` references, other CLAUDE.md files in the hierarchy, `.claude/rules/` files).

3. **Synthesize findings**:
   Merge results from all 3 agents into a single list grouped by severity. Deduplicate overlapping findings. Cross-reference against project-specific context the agents would not have (e.g., known issues where Claude ignores specific rules, recently changed conventions). See [multi-perspective-review.md](multi-perspective-review.md) for disagreement handling.

4. **Estimate token impact**:
   - Use the token counts from the rules-reviewer's output
   - Flag files over 200 lines as candidates for splitting into scoped rules
   - Flag total token cost if it seems disproportionate

5. **Present findings**:
   Group results by severity (Blocking, Improvements, Suggestions) per [quality-checklist.md](quality-checklist.md). For each finding, state: what the issue is, which file and line/section it's in, what the fix would be.

6. **Review-fix loop**:
   - Fix Blocking and Improvements immediately; escalate only when the fix has multiple plausible approaches and no available context disambiguates, or the same finding recurs after a fix attempt.
   - Delegate fixes to a `rules-writer` subagent, then re-review with all 3 agents. Iterate until all pass or 4 cycles complete per [multi-perspective-review.md](multi-perspective-review.md).
