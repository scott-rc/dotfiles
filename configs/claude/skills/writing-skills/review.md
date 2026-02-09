# Review Operation

Evaluate a Claude Code skill against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the skill**:
   - If the user provides a path, use it directly
   - If the user provides a skill name, search for `<name>/SKILL.md` in `~/.claude/skills/` and the project's skill directory
   - If neither, list available skills and ask the user to choose
   - Confirm the skill directory exists and contains a SKILL.md file

2. **Read all skill files**:
   - Read SKILL.md first
   - Read every `.md` file linked from SKILL.md (operations and references)
   - Read any other `.md` files in the directory to check for orphans
   - Read any scripts in `scripts/` if present

3. **Validate structure against spec**:
   Check each rule in [spec.md](spec.md):
   - Frontmatter: `name` and `description` present, name matches directory
   - Naming: all file names lowercase with hyphens, max 64 chars
   - SKILL.md body: has Operations section, each operation links to a file
   - Operation files: H1 matches SKILL.md, has numbered steps
   - Reference files: not referenced by other reference files
   - No orphan files (every file referenced from somewhere)

4. **Evaluate content quality against checklist**:
   Run through every item in [quality-checklist.md](quality-checklist.md):
   - Core quality (description, line count, terminology, progressive disclosure, examples)
   - Structure (frontmatter, naming, sections, file organization)
   - Content efficiency (token justification, redundancy, over-explaining, conciseness)
   - Scripts if applicable (error handling, constants, dependencies, paths)
   - Workflow quality (sequential steps, decision points, error cases, feedback)
   - Anti-patterns (nested references, vague names, Windows paths, time-sensitive content, inconsistent terms)

5. **Check for additional anti-patterns**:
   - SKILL.md contains step-by-step instructions instead of routing to operation files
   - Operation files that duplicate content from other operation files
   - Reference files that contain operation logic (numbered steps telling the agent what to do)
   - Description field that is too vague to trigger on user intent
   - Missing combined operations when multiple operations could logically be chained
   - Operation presents the user with multiple approaches when a single default with an escape hatch would be clearer

6. **Estimate token usage**:
   - Count approximate tokens for each file (rough: 1 token per 4 characters)
   - Flag files over 2000 tokens as candidates for splitting
   - Flag total skill size over 5000 tokens as potentially too large for SKILL.md

7. **Present findings**:
   Group results by severity:

   **Blocking** (must fix):
   - Missing required frontmatter fields
   - Broken file links
   - Missing operation files
   - Anti-patterns from the checklist

   **Improvements** (should fix):
   - Vague description lacking trigger keywords
   - Missing error handling in operations
   - Redundant content between files
   - Missing combined operations section

   **Suggestions** (nice to have):
   - Better file naming
   - Additional examples
   - Token optimization opportunities

   For each finding, state:
   - What the issue is
   - Which file it's in
   - What the fix would be (specific, not vague)

8. **Offer to apply fixes**:
   - Ask the user if they want to fix blocking issues automatically
   - For improvements and suggestions, list them and let the user choose which to apply
   - Apply fixes one at a time, confirming each change
