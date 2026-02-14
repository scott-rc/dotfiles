# Authoring Specification

Rules for authoring Claude Code agent skills and project rules (CLAUDE.md files). All operations in this skill validate against these rules.

## Shared Rules

These rules apply to both skills and rules files.

### Keyword Conventions

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in instruction files are used as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119):

- **MUST** / **MUST NOT** — absolute requirement or prohibition. The instruction is broken if violated.
- **SHOULD** / **SHOULD NOT** — strong recommendation. Can be ignored only with good reason.
- **MAY** — truly optional. No justification needed to omit.

Write these keywords in ALL CAPS when used with their RFC meaning. Use lowercase for ordinary prose. Reserve MUST for rules where violation causes failure — overuse dilutes its authority.

### Content Rules

- **Context window is a public good**: MUST only add information Claude does not already have. Challenge each line: does this teach something new, or does it restate common knowledge? Every token MUST justify its cost.
- **No time-sensitive information**: MUST NOT reference specific versions, dates, or URLs that will rot
- **Consistent terminology**: MUST pick one term and use it everywhere (e.g., "operation" not sometimes "command" and sometimes "action")
- **POSIX paths**: MUST use forward slashes. No backslashes, no Windows paths.
- **Markdown only**: All instruction files MUST be markdown. Use code blocks for shell commands.
- **RFC 2119 keywords**: SHOULD use MUST, SHOULD, and MAY (capitalized) per the Keyword Conventions section above. Reserve MUST for rules where violation breaks the outcome.

## Skill Specification

Rules specific to authoring Claude Code skills.

### Naming

- **Skill name**: MUST be lowercase, hyphens only, max 64 characters. SHOULD use gerund form when it reads naturally (e.g., `instructing-agents`, `managing-deploys`). MUST describe the skill's domain, not a single action.
  - MUST NOT contain XML tags
  - MUST NOT include reserved words: `anthropic`, `claude`
- **File names**: MUST be lowercase, hyphens only. MUST be descriptive of content (e.g., `commit-guidelines.md`, not `guidelines.md`). Operation files MUST be named after the operation (e.g., `create-skill.md`, `review-skill.md`).
- **Directory name**: MUST match the skill name exactly.

### Frontmatter

SKILL.md MUST start with YAML frontmatter delimited by `---`:

```yaml
---
name: <skill-name>              # Required. Must match directory name.
description: <text>             # Required. See Description Rules below.
---
```

Only `name` and `description` are recognized frontmatter keys. MUST NOT add custom keys.

### Description Rules

The `description` field is how Claude discovers and matches the skill to user intent. It MUST:

1. **MUST use third person**: "Handles..." / "Creates..." / "Reviews..." -- not "Handle" or "I handle"
2. **MUST state what AND when**: First clause says what it does, second says when to use it
3. **MUST include trigger keywords**: Verbs and nouns a user would naturally say (e.g., "commit, push, rebase" for a git skill)
4. **MUST be a single sentence**: One sentence, no line breaks, no bullet points
5. **MUST stay under 1024 characters**: Long descriptions get truncated in skill listings
6. **MUST NOT contain XML tags**

### SKILL.md Body

The body after frontmatter is the hub that routes to operation files. Constraints:

- **Max 500 lines / 5000 tokens**: MUST NOT exceed these limits. Anything longer should be split into referenced files.
- **Heading structure**: MUST have one H1 (the skill title), then H2 for sections, H3 for individual operations
- **Required sections**: MUST have "Operations" (H2) listing each operation with a one-line summary and a link to its file
- **Optional sections**: MAY have "Combined Operations" (H2) for multi-operation intent mapping, "References" (H2) for shared reference files
- **No inline instructions**: MUST route, not instruct. Keep operation details in their own files.

### Operation Files

Each operation file (e.g., `create-skill.md`, `review-skill.md`) contains the full instructions for one operation.

- **H1 heading**: MUST start with the operation name from SKILL.md (e.g., `# Commit Operation` for a `### Commit` entry)
- **Summary line**: MUST have one sentence after the heading describing what the operation does
- **Numbered steps**: MUST use numbered steps where each step has a **bold step name** followed by the instructions
- **Cross-references**: MUST use markdown links to reference files (e.g., `[spec.md](spec.md)`)
- **Self-contained**: SHOULD be understandable from the operation file alone (referenced files provide detail, not essential context)

### Reference Files

Reference files contain shared knowledge used by multiple operations (patterns, guidelines, templates, checklists).

- **One level deep**: SKILL.md and operation files can reference these files. Reference files MUST NOT reference other reference files.
- **Descriptive names**: MUST describe the content type (e.g., `commit-guidelines.md`, `git-patterns.md`)
- **No operation logic**: MUST provide information, not step-by-step instructions
- **Table of contents**: SHOULD include a table of contents for reference files over 100 lines

### Directory Structure

```
<skill-name>/
├── SKILL.md           # Hub (required)
├── <operation>.md     # One per operation (required, at least one)
├── <reference>.md     # Shared knowledge (optional)
├── scripts/           # Executable scripts (optional)
├── references/        # Additional reference material (optional, for large skills)
└── assets/            # Non-text files (optional)
```

Subdirectories are optional and only needed when the skill has many files of a given type.

### Skill Content Rules

These rules supplement the shared Content Rules above:

- **MCP tool names**: SHOULD use fully qualified `ServerName:tool_name` format when referencing MCP tools
- **Progressive disclosure**: MUST follow progressive disclosure — SKILL.md is concise, operation files are detailed, reference files go deep
- **Degrees of freedom**: SHOULD match instruction specificity to the task. High freedom (prose, multiple valid approaches) for creative/variable tasks. Medium freedom (pseudocode with parameters) when a preferred pattern exists. Low freedom (exact scripts, few parameters) for fragile/critical operations.

## Rules Specification

Rules for authoring CLAUDE.md project instructions and `.claude/rules/` scoped rules.

### Overview

CLAUDE.md files provide persistent instructions that Claude loads into every conversation. They configure Claude's behavior for a project or globally. Scoped rules (`.claude/rules/*.md`) provide path-specific instructions that only load when relevant files are being edited.

### File Locations

| File | Scope | When Loaded |
|------|-------|-------------|
| `CLAUDE.md` (project root) | Project-wide | Every conversation in that project |
| `CLAUDE.md` (subdirectory) | Subtree | When working on files in that subtree |
| `~/.claude/CLAUDE.md` | Global (user) | Every conversation across all projects |
| `.claude/rules/*.md` | Scoped | When `paths:` frontmatter matches active files |

CLAUDE.md files cascade: global, then project root, then subdirectories. More specific files supplement, not override, broader ones.

### Structure

- **Headings**: SHOULD use H1 for the file title, H2 for major sections. Keep the heading hierarchy flat.
- **@file references**: Use `@filename` to include content from other files (e.g., `@README.md`). Claude reads the referenced file as additional context. Place these near the top of the file.
- **Section separators**: Use `---` (horizontal rule) to visually separate distinct instruction groups.
- **Brevity**: MUST be concise. Every line loads into every conversation — verbosity has a direct token cost.

### Content Guidelines

**What to include:**
- Project-specific conventions Claude cannot infer (e.g., "use bun, not npm", "tests go in `__tests__/`")
- Build, test, and lint commands (e.g., "run `make check` before committing")
- Architectural decisions and constraints (e.g., "this is a monorepo with packages in `packages/`")
- File organization patterns (e.g., "one component per file, co-locate tests")
- References to key documentation via `@file` (e.g., `@README.md`, `@CONTRIBUTING.md`)

**What NOT to include:**
- Common knowledge Claude already has (language syntax, standard library usage, well-known framework patterns)
- Information already in referenced files (don't duplicate `@README.md` content)
- Time-sensitive content (version numbers, dates, URLs that may rot)
- Vague guidance ("write clean code", "follow best practices") — be specific or omit

### @file References

`@filename` tells Claude to read another file as additional context. Use it to:

- Pull in README, CONTRIBUTING, or architecture docs without duplicating them
- Reference style guides, API schemas, or other living documents

Syntax: `@path/to/file` on its own line or inline. Paths are relative to the CLAUDE.md file's location.

SHOULD prefer `@file` over copying content. If the source file changes, the reference stays current.

### Scoped Rules

Files in `.claude/rules/` with YAML `paths:` frontmatter only load when matching files are active:

```yaml
---
paths:
  - "src/api/**"
  - "*.test.ts"
---

Instructions that only apply when working on API files or tests.
```

Use scoped rules when instructions:
- Apply to a subset of the codebase, not the whole project
- Would add noise to the main CLAUDE.md for most conversations
- Are specific to a file type, directory, or feature area

### Anti-patterns

- **Duplicating README content**: Use `@README.md` instead of copying setup instructions
- **Common knowledge**: Don't teach Claude things it already knows (e.g., "use `git add` before `git commit`")
- **Vague instructions**: "Follow best practices" is not actionable. State the specific practice.
- **Excessive length**: If CLAUDE.md exceeds ~200 lines, split into scoped rules or use `@file` references
- **Unstable references**: Don't hardcode version numbers, specific dates, or URLs that may change
