# Review Skill

Evaluate a Claude Code skill against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If neither, discover available skills and present them as AskUserQuestion options
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Evaluate skill via subagent**:
   Spawn a Task subagent (type: skill-reviewer) with the skill directory path. The skill-reviewer agent reads all files, validates structure, evaluates content quality, and checks for anti-patterns. It returns findings grouped by severity with per-file token counts.

3. **Review skill-reviewer findings**:
   The skill-reviewer has already evaluated structure, content quality, and anti-patterns. MUST cross-reference findings against any project-specific context the agent would not have (e.g., project conventions from CLAUDE.md, skill interdependencies). Only re-read individual files inline when a finding needs verification.

4. **Estimate token usage**:
   - Use the token counts from the skill-reviewer's output
   - Flag files over 2000 tokens as candidates for splitting
   - Flag total skill size over 5000 tokens as potentially too large for SKILL.md

5. **Present findings**:
   Group results by severity:

   **Blocking** (MUST fix):
   - Missing required frontmatter fields
   - Broken file links
   - Missing operation files
   - Anti-patterns from the checklist

   **Improvements** (SHOULD fix):
   - Vague description lacking trigger keywords
   - Missing error handling in operations
   - Redundant content between files
   - Missing combined operations section

   **Suggestions** (MAY fix):
   - Better file naming
   - Additional examples
   - Token optimization opportunities

   For each finding, state:
   - What the issue is
   - Which file it's in
   - What the fix would be (specific, not vague)

6. **Offer to apply fixes**:
   - MUST present blocking fixes via AskUserQuestion before applying them
   - SHOULD present improvements and suggestions as AskUserQuestion options for the user to select
   - MUST apply fixes one at a time, confirming each change via AskUserQuestion
