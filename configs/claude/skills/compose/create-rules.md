# Create Rules

Write a CLAUDE.md or `.claude/rules/` rules file, producing clear and concise project instructions that configure Claude's behavior.

## Instructions

1. **Gather requirements**:
   Ask the user about:
   - What type of rules file they need (project CLAUDE.md, global CLAUDE.md, CLAUDE.local.md, unconditional rule, scoped rule, or user-level rule)
   - What project or directory the rules are for
   - What instructions or conventions they want to encode
   - Whether existing documentation (README, CONTRIBUTING, etc.) should be referenced via `@file`
   - If scoped: which file paths or patterns the rules should apply to
   - If personal/private: whether `CLAUDE.local.md` (per-project, auto-gitignored) or `~/.claude/rules/` (cross-project) is more appropriate

2. **Determine file location and type**:
   - **Project CLAUDE.md**: `<project-root>/CLAUDE.md` or `<project-root>/.claude/CLAUDE.md` — for project-wide instructions
   - **Subdirectory CLAUDE.md**: `<project-root>/<subdir>/CLAUDE.md` — for subtree-specific instructions
   - **CLAUDE.local.md**: `<project-root>/CLAUDE.local.md` — private per-project instructions, auto-added to .gitignore
   - **Global CLAUDE.md**: `~/.claude/CLAUDE.md` — for user-wide preferences across all projects
   - **Unconditional rule**: `<project-root>/.claude/rules/<name>.md` (no `paths:` frontmatter) — for topic-specific project instructions that always load
   - **Scoped rule**: `<project-root>/.claude/rules/<name>.md` (with `paths:` frontmatter) — for path-specific instructions
   - **User-level rule**: `~/.claude/rules/<name>.md` — for personal rules across all projects (loaded before project rules)
   - **Managed policy**: `/Library/Application Support/ClaudeCode/CLAUDE.md` (macOS) — organization-wide, requires IT/DevOps deployment
   - Rules files MAY be organized into subdirectories (e.g., `.claude/rules/frontend/react.md`)
   - MUST confirm the location with the user -- present 1-3 applicable locations via AskUserQuestion based on the user's requirements (e.g., project CLAUDE.md, `.claude/rules/<name>.md`, `~/.claude/rules/<name>.md`)
   - If a file already exists at the target location, read it and present options via AskUserQuestion: "Replace existing", "Extend existing", "Pick a different location"

3. **Assess existing documentation**:
   Spawn a Task subagent (type: Explore, model: haiku) to scan the project for existing documentation. The subagent MUST:
   - Check for README.md, CONTRIBUTING.md, and other docs in the project
   - Identify candidates for `@file` references to avoid duplicating content
   - Check for existing CLAUDE.md files in parent/child directories and `.claude/rules/` files
   - Return a list of relevant files with paths and brief content summaries

4. **Write the rules file**:
   - MUST follow [rules-spec.md](rules-spec.md) and [shared-rules.md](shared-rules.md)
   - MUST use the appropriate template from [rules-template.md](rules-template.md)
   - MUST use `@file` references for existing documentation instead of duplicating content
   - MUST write only instructions that teach Claude something it cannot infer on its own
   - MUST apply the conciseness test: for each line, ask "Would removing this cause Claude to make mistakes?" Cut lines that fail this test.
   - For scoped rules: MUST include `paths:` YAML frontmatter with glob patterns (brace expansion supported, e.g., `*.{ts,tsx}`)
   - For unconditional rules: MUST NOT include `paths:` frontmatter
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
