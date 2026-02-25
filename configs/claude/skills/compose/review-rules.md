# Review Rules

Evaluate a CLAUDE.md or scoped rules file against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the rules file(s)**:
   - If the user provides a path, use it directly
   - If the user says "review my CLAUDE.md", check the current project root first, then `~/.claude/CLAUDE.md`
   - If neither, discover CLAUDE.md files in the project and present them as AskUserQuestion options
   - SHOULD also identify related files: other CLAUDE.md files in parent/child directories, `.claude/rules/` files (including subdirectories), `~/.claude/rules/` user-level rules

2. **Evaluate rules via subagent**:
   Spawn a Task subagent (type: rules-reviewer) with the rules file path. The rules-reviewer agent reads the target file and all related files (`@file` references, other CLAUDE.md files in the hierarchy, `.claude/rules/` files), validates structure, evaluates content quality, checks for anti-patterns, and returns findings grouped by severity with per-file token counts.

3. **Review rules-reviewer findings**:
   The rules-reviewer has already evaluated structure, content quality, and anti-patterns. MUST cross-reference findings against any project-specific context the agent would not have (e.g., known issues where Claude ignores specific rules, recently changed conventions). Only re-read individual files inline when a finding needs verification.

4. **Estimate token impact**:
   - Use the token counts from the rules-reviewer's output
   - Flag files over 200 lines as candidates for splitting into scoped rules
   - Flag total token cost if it seems disproportionate

5. **Present findings**:
   Group results by severity:

   **Blocking** (MUST fix):
   - Broken `@file` references
   - Content that duplicates referenced files
   - Common knowledge that wastes tokens
   - Vague, non-actionable instructions

   **Improvements** (SHOULD fix):
   - Instructions that belong in scoped rules
   - Missing `@file` references for existing documentation
   - Excessive length (over ~200 lines)
   - Missing useful sections (build commands, test commands, architecture)

   **Suggestions** (MAY fix):
   - Token optimization opportunities
   - Better section organization
   - Additional `@file` candidates

   For each finding, state:
   - What the issue is
   - Where it is (line number or section)
   - What the fix would be (specific, not vague)

6. **Offer to apply fixes**:
   - MUST present blocking fixes via AskUserQuestion before applying them
   - SHOULD present improvements and suggestions as AskUserQuestion options for the user to select
   - MUST apply fixes one at a time, confirming each change via AskUserQuestion
