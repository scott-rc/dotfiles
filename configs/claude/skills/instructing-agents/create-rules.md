# Create Rules

Write a CLAUDE.md or `.claude/rules/` rules file, producing clear and concise project instructions that configure Claude's behavior.

## Instructions

1. **Gather requirements**:
   Ask the user about:
   - What type of rules file they need (project CLAUDE.md, global CLAUDE.md, or scoped rule)
   - What project or directory the rules are for
   - What instructions or conventions they want to encode
   - Whether existing documentation (README, CONTRIBUTING, etc.) should be referenced via `@file`
   - If scoped: which file paths or patterns the rules should apply to

2. **Determine file location and type**:
   - **Project CLAUDE.md**: `<project-root>/CLAUDE.md` — for project-wide instructions
   - **Subdirectory CLAUDE.md**: `<project-root>/<subdir>/CLAUDE.md` — for subtree-specific instructions
   - **Global CLAUDE.md**: `~/.claude/CLAUDE.md` — for user-wide preferences across all projects
   - **Scoped rule**: `<project-root>/.claude/rules/<name>.md` — for path-specific instructions
   - MUST confirm the location with the user before proceeding
   - If a file already exists at the target location, read it and ask the user whether to replace, extend, or pick a different location

3. **Assess existing documentation**:
   - SHOULD check for README.md, CONTRIBUTING.md, and other docs in the project
   - SHOULD identify candidates for `@file` references to avoid duplicating content
   - SHOULD check for existing CLAUDE.md files in parent/child directories to avoid conflicts or redundancy

4. **Write the rules file**:
   - MUST follow the Rules Specification section of [spec.md](spec.md)
   - MUST use the appropriate template from [rules-template.md](rules-template.md)
   - MUST use `@file` references for existing documentation instead of duplicating content
   - MUST write only instructions that teach Claude something it cannot infer on its own
   - For scoped rules: MUST include `paths:` YAML frontmatter with glob patterns
   - SHOULD keep CLAUDE.md files under ~200 lines
   - SHOULD use `---` separators between distinct instruction groups

5. **Validate**:
   - MUST run through the Rules-specific items in [quality-checklist.md](quality-checklist.md)
   - MUST verify all `@file` references point to existing files
   - MUST check for content that duplicates referenced files
   - MUST check for common knowledge or vague instructions and remove them
   - MUST fix any issues found before reporting to the user

6. **Report results**:
   - MUST show the complete rules file content for user review
   - MUST list any `@file` references and confirm they resolve correctly
   - SHOULD note the approximate token cost (the file loads into every conversation)
   - If scoped: MUST show the `paths:` patterns and explain which files they match
