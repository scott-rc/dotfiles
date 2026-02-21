# Authoring Specification

Rules for authoring Claude Code agent skills and rules files (CLAUDE.md, `.claude/rules/`). All operations in this skill validate against these rules.

## Shared Rules

These rules apply to both skills and rules files.

### Keyword Conventions

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in skill and rules files are used as described in RFC 2119:

- **MUST** / **MUST NOT** — absolute requirement or prohibition. The instruction is broken if violated.
- **SHOULD** / **SHOULD NOT** — strong recommendation. Can be ignored only with good reason.
- **MAY** — truly optional. No justification needed to omit.

Write these keywords in ALL CAPS when used with their RFC meaning. Use lowercase for ordinary prose. Reserve MUST for rules where violation causes failure — overuse dilutes its authority.

### Content Rules

- **Context window is a public good**: MUST only add information Claude does not already have. Challenge each line: does this teach something new, or does it restate common knowledge? Every token MUST justify its cost.
- **Write tight**: SHOULD use terse, imperative prose. Drop articles, filler words, and hedging where meaning is preserved. Prefer sentence fragments in lists. Lead with the verb. Example: "MUST run linter before committing" not "You should make sure to run the linter tool before you commit your changes".
- **No time-sensitive information**: MUST NOT reference specific versions, dates, or URLs that will rot
- **Consistent terminology**: MUST pick one term and use it everywhere (e.g., "operation" not sometimes "command" and sometimes "action")
- **POSIX paths**: MUST use forward slashes. No backslashes, no Windows paths.
- **Markdown only**: All skill and rules files MUST be markdown. Use code blocks for shell commands.
- **No tables**: MUST use lists instead of markdown tables. Tables add significant token overhead (pipes, header separators, padding) with no benefit for LLM comprehension. Use bulleted lists with `—` separators for key-value pairs, or split into labeled sub-lists for multi-column data.
- **RFC 2119 keywords**: SHOULD use MUST, SHOULD, and MAY (capitalized) per the Keyword Conventions section above. Reserve MUST for rules where violation breaks the outcome.

## Skill Specification

Rules specific to authoring Claude Code skills.

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
- `allowed-tools` — comma-separated list of tools available when this skill is active (e.g., `Read, Grep, Glob`). Restricts the agent's capabilities during the skill.
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

Skill descriptions consume context budget (2% of context window, fallback 16,000 chars). Many skills with long descriptions can exceed the budget, causing some to be excluded from the agent's awareness. Keep descriptions concise. The env var `SLASH_COMMAND_TOOL_CHAR_BUDGET` overrides the default budget.

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
- **Task vs reference content**: Task skills give step-by-step instructions for a specific workflow (often paired with `disable-model-invocation: true`). Reference skills add knowledge Claude applies to current work (often paired with `user-invocable: false`). Shape content to match the invocation pattern.
- **Interview before assumptions**: Operations that act on user intent (creating, configuring, scaffolding) SHOULD begin with an interview step that gathers requirements before proceeding. The interview SHOULD ask only enough to unblock the next decision, use follow-up rounds for complexity revealed by initial answers, and summarize understanding for user confirmation before acting. Operations MUST NOT silently assume requirements the user hasn't stated when multiple valid options exist.
- **Cross-skill delegation**: When an operation needs to run another skill's workflow or load another skill's references, MUST use the Skill tool to invoke that skill (`skill: "<name>", args: "<routing context>"`). MUST NOT reference another skill's files via relative paths (e.g., `../other-skill/file.md`) — cross-skill file references are unreliable because the other skill's context (routing, templates, transitive references) is not formally loaded. Using the Skill tool loads the other skill's SKILL.md with its full context.

## Rules Specification

Rules for authoring CLAUDE.md and `.claude/rules/` rules files.

### Overview

CLAUDE.md files provide persistent instructions that Claude loads into every conversation. They configure Claude's behavior for a project or globally. Rules files (`.claude/rules/*.md`) provide modular, topic-specific project instructions — either unconditional or scoped to specific file paths.

### File Locations

- `CLAUDE.md` or `.claude/CLAUDE.md` (project root) — Project-wide, loads every conversation in that project
- `CLAUDE.md` (subdirectory) — Subtree, loads when working on files in that subtree
- `CLAUDE.local.md` (project root) — Private per-project instructions, auto-added to .gitignore. Use for personal preferences that shouldn't be checked in (local dev URLs, personal test data, sandbox credentials).
- `~/.claude/CLAUDE.md` — Global (user), loads every conversation across all projects
- `.claude/rules/*.md` (no `paths:` frontmatter) — Unconditional project rules, always loaded with the same priority as `.claude/CLAUDE.md`
- `.claude/rules/*.md` (with `paths:` frontmatter) — Scoped rules, loads only when matching files are active
- `~/.claude/rules/*.md` — User-level rules, loaded before project rules across all projects
- Managed policy (macOS: `/Library/Application Support/ClaudeCode/CLAUDE.md`) — Organization-wide policy, requires IT/DevOps deployment

CLAUDE.md files cascade: global, then project root, then subdirectories. More specific files supplement, not override, broader ones. User-level rules load before project rules, giving project rules higher priority.

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

#### Conciseness Test

For each line, ask: "Would removing this cause Claude to make mistakes?" If not, cut it. If Claude keeps ignoring a rule, the file is probably too long — prune aggressively rather than adding more rules. Treat CLAUDE.md like code: review when things go wrong, prune regularly.

#### Emphasis

You can tune instruction adherence by adding emphasis ("IMPORTANT", "YOU MUST"). Use sparingly — if everything is important, nothing is.

### @file References

`@filename` tells Claude to read another file as additional context. Use it to:

- Pull in README, CONTRIBUTING, or architecture docs without duplicating them
- Reference style guides, API schemas, or other living documents

Syntax: `@path/to/file` on its own line or inline. Both relative and absolute paths are allowed; relative paths resolve from the file's location. Imported files can recursively import other files (max depth 5 hops). Imports are not evaluated inside markdown code spans or code blocks.

SHOULD prefer `@file` over copying content. If the source file changes, the reference stays current.

### Modular Rules (`.claude/rules/`)

All `.md` files in `.claude/rules/` are automatically discovered, including in subdirectories (recursive). Symlinks are supported and resolved normally.

**Unconditional rules** — files without `paths:` frontmatter load every conversation, same priority as `.claude/CLAUDE.md`. Use these to split a large CLAUDE.md into focused, topic-specific files.

**Scoped rules** — files with `paths:` frontmatter only load when matching files are active:

```yaml
---
paths:
  - "src/api/**"
  - "*.test.ts"
---

Instructions that only apply when working on API files or tests.
```

Glob patterns support brace expansion: `"src/**/*.{ts,tsx}"` matches both `.ts` and `.tsx` files. Multiple patterns can be listed.

**Subdirectory organization** — rules can be grouped into subdirectories for structure:

```
.claude/rules/
├── frontend/
│   ├── react.md
│   └── styles.md
├── backend/
│   ├── api.md
│   └── database.md
└── general.md
```

Use unconditional rules when:
- CLAUDE.md exceeds ~200 lines and needs to be split into focused topics
- Instructions are project-wide but logically separate from the main CLAUDE.md

Use scoped rules when instructions:
- Apply to a subset of the codebase, not the whole project
- Would add noise to the main CLAUDE.md for most conversations
- Are specific to a file type, directory, or feature area

### Anti-patterns

- **Duplicating README content**: Use `@README.md` instead of copying setup instructions
- **Common knowledge**: Don't teach Claude things it already knows (e.g., "use `git add` before `git commit`")
- **Vague instructions**: "Follow best practices" is not actionable. State the specific practice.
- **Excessive length**: If CLAUDE.md exceeds ~200 lines, split into `.claude/rules/` files or use `@file` references
- **Unstable references**: Don't hardcode version numbers, specific dates, or URLs that may change
- **Over-specified files**: If CLAUDE.md is so long that Claude ignores rules, it needs aggressive pruning, not more rules. Adding emphasis to every instruction is a symptom of this problem.
- **Kitchen-sink context**: Adding unrelated instructions to a single file reduces effectiveness. Split by topic or scope instead.
