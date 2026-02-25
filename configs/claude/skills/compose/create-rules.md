# Create Rules

Write a CLAUDE.md or `.claude/rules/` rules file, producing clear and concise project instructions that configure Claude's behavior.

## Instructions

1. **Gather requirements**:
   Ask the user via AskUserQuestion about:
   - What type of rules file they need -- present as AskUserQuestion options: "Project CLAUDE.md", "Global CLAUDE.md", "CLAUDE.local.md", "Unconditional rule", "Scoped rule", "User-level rule"
   - What project or directory the rules are for
   - What instructions or conventions they want to encode
   - Whether existing documentation (README, CONTRIBUTING, etc.) should be referenced via `@file`
   - If scoped: which file paths or patterns the rules should apply to
   - If personal/private: present as AskUserQuestion options: "CLAUDE.local.md" (description: per-project, auto-gitignored), "~/.claude/rules/" (description: cross-project)

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

4. **Write and verify**:
   Delegate to `rules-writer` (Task agent) with:
   - `mode` — `create`, `replace`, or `extend` (from step 2)
   - `file_path` — target path (from step 2)
   - `file_type` — one of: `project-claude-md`, `global-claude-md`, `claude-local-md`, `unconditional-rule`, `scoped-rule`, `user-rule`
   - `scope_patterns` — glob patterns (scoped rules only, from step 1)
   - `requirements` — user's content requirements (from step 1)
   - `existing_content` — current file content (extend mode only, from step 2)
   - `existing_docs` — discovered docs with summaries (from step 3)

   The agent writes the file, verifies structure and quality, and self-corrects up to 3 times.

5. **Report results**:
   Read the agent's output and present to the user:
   - MUST show the complete rules file content for review
   - MUST list `@file` references and their resolution status
   - SHOULD note the approximate token cost (the file loads into every conversation)
   - If scoped: MUST show the `paths:` patterns and explain which files they match
