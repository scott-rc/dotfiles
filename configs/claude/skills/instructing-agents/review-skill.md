# Review Skill

Evaluate a Claude Code skill against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If neither, list available skills and ask the user to choose
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Read all skill files**:
   - MUST read SKILL.md first
   - MUST read every `.md` file linked from SKILL.md (operations and references)
   - MUST check for orphan `.md` files in the directory not linked from anywhere
   - SHOULD read any scripts in `scripts/` if present

3. **Validate structure against spec**:
   MUST validate the skill against every rule in the Skill Specification section of [spec.md](spec.md), covering frontmatter, naming, SKILL.md body, operation files, reference files, and orphan files.

4. **Evaluate content quality against checklist**:
   MUST evaluate against every item in [quality-checklist.md](quality-checklist.md).

5. **Check for additional anti-patterns** not covered by the checklist:
   - Operation files that duplicate content from other operation files
   - Reference files that contain operation logic (numbered steps telling the agent what to do)
   - Missing combined operations when multiple operations could logically be chained

6. **Estimate token usage**:
   - Count approximate tokens for each file (rough: 1 token per 4 characters)
   - Flag files over 2000 tokens as candidates for splitting
   - Flag total skill size over 5000 tokens as potentially too large for SKILL.md

7. **Present findings**:
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

8. **Offer to apply fixes**:
   - MUST ask the user about blocking fixes before applying them
   - SHOULD list improvements and suggestions for the user to choose from
   - MUST apply fixes one at a time, confirming each change
