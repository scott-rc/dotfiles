# Skill Specification

Rules for authoring Claude Code agent skills. All operations in this skill validate against these rules.

## Naming

- **Skill name**: Lowercase, hyphens only. Max 64 characters. Use gerund form when it reads naturally (e.g., `writing-skills`, `managing-deploys`). The name must describe the skill's domain, not a single action.
  - Must not contain XML tags
  - Must not include reserved words: `anthropic`, `claude`
- **File names**: Lowercase, hyphens only. Descriptive of content (e.g., `commit-guidelines.md`, not `guidelines.md`). Operation files are named after the operation (e.g., `create.md`, `review.md`).
- **Directory name**: Must match the skill name exactly.

## Frontmatter

SKILL.md must start with YAML frontmatter delimited by `---`:

```yaml
---
name: <skill-name>              # Required. Must match directory name.
description: <text>             # Required. See Description Rules below.
---
```

Only `name` and `description` are recognized frontmatter keys. Do not add custom keys.

## Description Rules

The `description` field is how Claude discovers and matches the skill to user intent. It must:

1. **Use third person**: "Handles..." / "Creates..." / "Reviews..." -- not "Handle" or "I handle"
2. **State what AND when**: First clause says what it does, second says when to use it
3. **Include trigger keywords**: Verbs and nouns a user would naturally say (e.g., "commit, push, rebase" for a git skill)
4. **Be a single sentence**: One sentence, no line breaks, no bullet points
5. **Stay under 1024 characters**: Long descriptions get truncated in skill listings
6. **No XML tags**: The description must not contain XML-style tags

## SKILL.md Body

The body after frontmatter is the hub that routes to operation files. Constraints:

- **Max 500 lines / 5000 tokens**: Anything longer should be split into referenced files
- **Heading structure**: One H1 (the skill title), then H2 for sections, H3 for individual operations
- **Required sections**: "Operations" (H2) listing each operation with a one-line summary and a link to its file
- **Optional sections**: "Combined Operations" (H2) for multi-operation intent mapping, "References" (H2) for shared reference files
- **No inline instructions**: SKILL.md should route, not instruct. Keep operation details in their own files.

## Operation Files

Each operation file (e.g., `create.md`, `review.md`) contains the full instructions for one operation.

- **H1 heading**: Must start with the operation name from SKILL.md (e.g., `# Commit Operation` for a `### Commit` entry)
- **Summary line**: One sentence after the heading describing what the operation does
- **Numbered steps**: Each step has a **bold step name** followed by the instructions
- **Cross-references**: Use markdown links to reference files (e.g., `[spec.md](spec.md)`)
- **Self-contained**: A reader should understand the full operation from the operation file alone (referenced files provide detail, not essential context)

## Reference Files

Reference files contain shared knowledge used by multiple operations (patterns, guidelines, templates, checklists).

- **One level deep**: SKILL.md and operation files can reference these files. Reference files must NOT reference other reference files.
- **Descriptive names**: Name describes the content type (e.g., `commit-guidelines.md`, `git-patterns.md`)
- **No operation logic**: Reference files provide information, not step-by-step instructions
- **Table of contents**: Include a table of contents for reference files over 100 lines

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

- **Context window is a public good**: Only add information Claude does not already have. Challenge each line: does this teach something new, or does it restate common knowledge? Every token in a skill must justify its cost.
- **No time-sensitive information**: Don't reference specific versions, dates, or URLs that will rot
- **Consistent terminology**: Pick one term and use it everywhere (e.g., "operation" not sometimes "command" and sometimes "action")
- **POSIX paths**: Use forward slashes. No backslashes, no Windows paths.
- **Markdown only**: All instruction files must be markdown. Use code blocks for shell commands.
- **MCP tool names**: Use fully qualified `ServerName:tool_name` format when referencing MCP tools
- **Progressive disclosure**: SKILL.md is concise, operation files are detailed, reference files go deep
- **Degrees of freedom**: Match instruction specificity to the task. High freedom (prose, multiple valid approaches) for creative/variable tasks. Medium freedom (pseudocode with parameters) when a preferred pattern exists. Low freedom (exact scripts, few parameters) for fragile/critical operations.
