# Review Rules

Evaluate a CLAUDE.md or scoped rules file against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the rules file(s)**:
   - If the user provides a path, use it directly
   - If the user says "review my CLAUDE.md", check the current project root first, then `~/.claude/CLAUDE.md`
   - If neither, list CLAUDE.md files found in the project and ask the user to choose
   - SHOULD also identify related files: other CLAUDE.md files in parent/child directories, `.claude/rules/` files

2. **Read all related files**:
   - MUST read the target rules file
   - MUST read every file referenced via `@filename`
   - MUST read other CLAUDE.md files in the project hierarchy (parent dirs, subdirs) to check for conflicts or redundancy
   - SHOULD read `.claude/rules/` files if they exist

3. **Validate against spec**:
   MUST validate the rules file against every rule in the Rules Specification section of [spec.md](spec.md), covering file location, structure, content guidelines, `@file` references, scoped rules frontmatter, and anti-patterns.

4. **Evaluate content quality against checklist**:
   MUST run through the Rules-specific items in [quality-checklist.md](quality-checklist.md):
   - Structure — Rules (@file references, content duplication, scoped rules paths, heading hierarchy)
   - Content efficiency (token justification, redundancy, novel information, actionable instructions)
   - Rules quality (granularity, common knowledge, README duplication, stable content, scope placement)
   - Rules anti-patterns (duplicated content, vague instructions, common knowledge, excessive length)

5. **Check for additional issues**:
   - Instructions that belong in scoped rules rather than the main CLAUDE.md (they apply to a small subset of files)
   - Missing sections that would help Claude (e.g., no build/test commands, no architecture overview)
   - `@file` references to files that contain mostly irrelevant content (should extract relevant parts instead)
   - Conflicts or redundancy with other CLAUDE.md files in the project hierarchy

6. **Estimate token impact**:
   - Count approximate tokens (rough: 1 token per 4 characters) for the rules file itself
   - Count tokens for all `@file` referenced files (these also load into context)
   - Report total token cost per conversation
   - Flag files over 200 lines as candidates for splitting into scoped rules

7. **Present findings**:
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

8. **Offer to apply fixes**:
   - MUST ask the user about blocking fixes before applying them
   - SHOULD list improvements and suggestions for the user to choose from
   - MUST apply fixes one at a time, confirming each change
