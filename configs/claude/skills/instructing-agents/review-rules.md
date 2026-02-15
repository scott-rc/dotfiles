# Review Rules

Evaluate a CLAUDE.md or scoped rules file against best practices, report findings grouped by severity, and offer to fix issues.

## Instructions

1. **Locate the rules file(s)**:
   - If the user provides a path, use it directly
   - If the user says "review my CLAUDE.md", check the current project root first, then `~/.claude/CLAUDE.md`
   - If neither, list CLAUDE.md files found in the project and ask the user to choose
   - SHOULD also identify related files: other CLAUDE.md files in parent/child directories, `.claude/rules/` files (including subdirectories), `~/.claude/rules/` user-level rules

2. **Read all related files**:
   - MUST read the target rules file
   - MUST read every file referenced via `@filename`
   - MUST read other CLAUDE.md files in the project hierarchy (parent dirs, subdirs) to check for conflicts or redundancy
   - MUST read `.claude/rules/` files if they exist (including subdirectories — all `.md` files are discovered recursively)

3. **Validate against spec**:
   MUST validate the rules file against every rule in the Rules Specification section of [spec.md](spec.md), covering file location, structure, content guidelines, `@file` references, scoped rules frontmatter, and anti-patterns.

4. **Evaluate content quality against checklist**:
   MUST evaluate against every Rules-relevant item in [quality-checklist.md](quality-checklist.md).

5. **Check for additional issues**:
   - Instructions that belong in scoped rules rather than the main CLAUDE.md (they apply to a small subset of files)
   - Instructions that could be split from CLAUDE.md into unconditional `.claude/rules/` files for better organization (especially if CLAUDE.md exceeds ~200 lines)
   - Missing sections that would help Claude (e.g., no build/test commands, no architecture overview)
   - `@file` references to files that contain mostly irrelevant content (should extract relevant parts instead)
   - Conflicts or redundancy with other CLAUDE.md files in the project hierarchy or with `.claude/rules/` files
   - Rules files in `.claude/rules/` without `paths:` that should be scoped, or scoped rules with overly broad patterns

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
