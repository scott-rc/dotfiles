# Skill Specification

Rules for authoring Claude Code agent skills. All operations in this skill validate against these rules.

## Keyword Conventions

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in skill files are used as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119):

- **MUST** / **MUST NOT** — absolute requirement or prohibition. The skill is broken if violated.
- **SHOULD** / **SHOULD NOT** — strong recommendation. Can be ignored only with good reason.
- **MAY** — truly optional. No justification needed to omit.

Write these keywords in ALL CAPS when used with their RFC meaning. Use lowercase for ordinary prose. Reserve MUST for rules where violation causes skill failure — overuse dilutes its authority.

## Naming

- **Skill name**: MUST be lowercase, hyphens only, max 64 characters. SHOULD use gerund form when it reads naturally (e.g., `writing-skills`, `managing-deploys`). MUST describe the skill's domain, not a single action.
  - MUST NOT contain XML tags
  - MUST NOT include reserved words: `anthropic`, `claude`
- **File names**: MUST be lowercase, hyphens only. MUST be descriptive of content (e.g., `commit-guidelines.md`, not `guidelines.md`). Operation files MUST be named after the operation (e.g., `create.md`, `review.md`).
- **Directory name**: MUST match the skill name exactly.

## Frontmatter

SKILL.md MUST start with YAML frontmatter delimited by `---`:

```yaml
---
name: <skill-name>              # Required. Must match directory name.
description: <text>             # Required. See Description Rules below.
---
```

Only `name` and `description` are recognized frontmatter keys. MUST NOT add custom keys.

## Description Rules

The `description` field is how Claude discovers and matches the skill to user intent. It MUST:

1. **MUST use third person**: "Handles..." / "Creates..." / "Reviews..." -- not "Handle" or "I handle"
2. **MUST state what AND when**: First clause says what it does, second says when to use it
3. **MUST include trigger keywords**: Verbs and nouns a user would naturally say (e.g., "commit, push, rebase" for a git skill)
4. **MUST be a single sentence**: One sentence, no line breaks, no bullet points
5. **MUST stay under 1024 characters**: Long descriptions get truncated in skill listings
6. **MUST NOT contain XML tags**

## SKILL.md Body

The body after frontmatter is the hub that routes to operation files. Constraints:

- **Max 500 lines / 5000 tokens**: MUST NOT exceed these limits. Anything longer should be split into referenced files.
- **Heading structure**: MUST have one H1 (the skill title), then H2 for sections, H3 for individual operations
- **Required sections**: MUST have "Operations" (H2) listing each operation with a one-line summary and a link to its file
- **Optional sections**: MAY have "Combined Operations" (H2) for multi-operation intent mapping, "References" (H2) for shared reference files
- **No inline instructions**: MUST route, not instruct. Keep operation details in their own files.

## Operation Files

Each operation file (e.g., `create.md`, `review.md`) contains the full instructions for one operation.

- **H1 heading**: MUST start with the operation name from SKILL.md (e.g., `# Commit Operation` for a `### Commit` entry)
- **Summary line**: MUST have one sentence after the heading describing what the operation does
- **Numbered steps**: MUST use numbered steps where each step has a **bold step name** followed by the instructions
- **Cross-references**: MUST use markdown links to reference files (e.g., `[spec.md](spec.md)`)
- **Self-contained**: SHOULD be understandable from the operation file alone (referenced files provide detail, not essential context)

## Reference Files

Reference files contain shared knowledge used by multiple operations (patterns, guidelines, templates, checklists).

- **One level deep**: SKILL.md and operation files can reference these files. Reference files MUST NOT reference other reference files.
- **Descriptive names**: MUST describe the content type (e.g., `commit-guidelines.md`, `git-patterns.md`)
- **No operation logic**: MUST provide information, not step-by-step instructions
- **Table of contents**: SHOULD include a table of contents for reference files over 100 lines

## Directory Structure

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

## Content Rules

- **Context window is a public good**: MUST only add information Claude does not already have. Challenge each line: does this teach something new, or does it restate common knowledge? Every token in a skill MUST justify its cost.
- **No time-sensitive information**: MUST NOT reference specific versions, dates, or URLs that will rot
- **Consistent terminology**: MUST pick one term and use it everywhere (e.g., "operation" not sometimes "command" and sometimes "action")
- **POSIX paths**: MUST use forward slashes. No backslashes, no Windows paths.
- **Markdown only**: All instruction files MUST be markdown. Use code blocks for shell commands.
- **MCP tool names**: SHOULD use fully qualified `ServerName:tool_name` format when referencing MCP tools
- **Progressive disclosure**: MUST follow progressive disclosure — SKILL.md is concise, operation files are detailed, reference files go deep
- **Degrees of freedom**: SHOULD match instruction specificity to the task. High freedom (prose, multiple valid approaches) for creative/variable tasks. Medium freedom (pseudocode with parameters) when a preferred pattern exists. Low freedom (exact scripts, few parameters) for fragile/critical operations.
- **RFC 2119 keywords**: SHOULD use MUST, SHOULD, and MAY (capitalized) per the Keyword Conventions section above. Reserve MUST for rules where violation breaks the skill.
