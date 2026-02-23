# Skill Specification

Rules specific to authoring Claude Code skills. All operations in this skill validate against these rules.

## Instructions

These rules supplement the shared authoring rules.

### Naming

- **Skill name**: MUST be lowercase, hyphens only, max 64 characters. SHOULD use gerund form when it reads naturally (e.g., `managing-deploys`, `reviewing-code`). MUST describe the skill's domain, not a single action.
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

Required fields: `name` and `description`. Optional fields (include only when needed):

- `argument-hint` — autocomplete hint shown after skill name (e.g., `[issue-number]`)
- `disable-model-invocation: true` — prevents Claude from auto-loading the skill; only the user can invoke it via `/`. Use for side-effect workflows where unintended invocation would be harmful.
- `user-invocable: false` — hides the skill from the `/` menu; only Claude can load it. Use for background knowledge that should augment Claude's behavior without user invocation.
- `allowed-tools` — comma-separated list of tools available when this skill is active (e.g., `Read, Grep, Glob`). Restricts Claude's available tools during the skill.
- `model` — model override for this skill (`sonnet`, `opus`, `haiku`)
- `context: fork` — runs the skill in a subagent (see Subagent Execution below)
- `agent` — subagent type when `context: fork` is set (`Explore`, `Plan`, `general-purpose`, or a custom agent type)
- `hooks` — lifecycle hooks scoped to this skill

MUST NOT add keys not listed above.

### Description Rules

The `description` field is how Claude discovers and matches the skill to user intent. It MUST:

1. **MUST use third person**: "Handles..." / "Creates..." / "Reviews..." -- not "Handle" or "I handle"
2. **MUST state what AND when**: First clause says what it does, second says when to use it
3. **MUST include trigger keywords**: Verbs and nouns a user would naturally say (e.g., "commit, push, rebase" for a git skill)
4. **MUST be a single sentence**: One sentence, no line breaks, no bullet points
5. **MUST stay under 1024 characters**: Long descriptions get truncated in skill listings
6. **MUST NOT contain XML tags**

### Description Budget

Skill descriptions consume context budget (2% of context window, fallback 16,000 chars). Many skills with long descriptions can exceed the budget, causing some to be excluded from Claude's awareness. Keep descriptions concise. The env var `SLASH_COMMAND_TOOL_CHAR_BUDGET` overrides the default budget.

### String Substitutions

Skill content supports string substitutions that are resolved before the content reaches Claude:

- `$ARGUMENTS` — the full argument string passed after the skill name
- `$ARGUMENTS[N]` or `$N` — positional argument access (0-indexed)
- `${CLAUDE_SESSION_ID}` — unique ID for the current session

### Dynamic Context Injection

Use `` !`command` `` syntax to inject the output of a shell command into skill content. The command runs before the skill content reaches Claude, and the output replaces the placeholder inline. Use this to inject runtime data (git state, file listings, environment info) that would otherwise require an extra tool call.

### Subagent Execution

When `context: fork` is set in frontmatter, the skill runs in an isolated subagent context. The `agent` field selects the executor type (`Explore`, `Plan`, `general-purpose`, or custom). The skill content becomes the task prompt for the subagent.

This only makes sense for task-oriented skills that produce a result — reference content should not use `context: fork` because it needs to augment the main conversation context, not run in isolation.

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
- **Cross-references**: MUST use markdown links to reference files (e.g., `[commit-guidelines.md](commit-guidelines.md)`)
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

These rules supplement the shared Content Rules:

- **MCP tool names**: SHOULD use fully qualified `ServerName:tool_name` format when referencing MCP tools
- **Progressive disclosure**: MUST follow progressive disclosure — SKILL.md is concise, operation files are detailed, reference files go deep
- **Degrees of freedom**: SHOULD match instruction specificity to the task. High freedom (prose, multiple valid approaches) for creative/variable tasks. Medium freedom (pseudocode with parameters) when a preferred pattern exists. Low freedom (exact scripts, few parameters) for fragile/critical operations.
- **Task vs reference content**: Task skills give step-by-step instructions for a specific workflow (often paired with `disable-model-invocation: true`). Reference skills add knowledge Claude applies to current work (often paired with `user-invocable: false`). Shape content to match the invocation pattern.
- **Interview before assumptions**: Operations that act on user intent (creating, configuring, scaffolding) SHOULD begin with an interview step that gathers requirements before proceeding. The interview SHOULD ask only enough to unblock the next decision, use follow-up rounds for complexity revealed by initial answers, and summarize understanding for user confirmation before acting. Operations MUST NOT silently assume requirements the user hasn't stated when multiple valid options exist. When presenting choices or requesting values, SHOULD apply the confirmation pattern from [content-patterns.md](content-patterns.md): offer 1-3 idiomatic defaults as AskUserQuestion options rather than open-ended questions.
- **Cross-skill delegation**: When an operation needs to run another skill's workflow or load another skill's references, MUST use the Skill tool to invoke that skill (`skill: "<name>", args: "<routing context>"`). MUST NOT reference another skill's files via relative paths (e.g., `../other-skill/file.md`) — cross-skill file references are unreliable because the other skill's context (routing, templates, transitive references) is not formally loaded. Using the Skill tool loads the other skill's SKILL.md with its full context.
- **Subagent delegation**: Operations SHOULD delegate heavy I/O (reading many files, codebase exploration, multi-step analysis) to Task subagents rather than performing it inline. The orchestrator's context is shared across the entire session -- every file read or verbose result inline consumes budget that later operations need. Write targeted subagent prompts with all necessary context (file paths, criteria, conventions) so the subagent works autonomously. Choose appropriate subagent types (`Explore` for search/read, `general-purpose` for multi-step work) and models (`haiku` for simple reads, `sonnet` for analysis, `opus` for deep reasoning). Subagents SHOULD return concise results -- not raw file contents.
