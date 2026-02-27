# Skill Specification

Rules specific to authoring Claude Code skills. All operations in this skill validate against these rules.

## Mental Model

```
SKILL.md (router)
├── Inline operations (linear, no refs, no branching)
├── Operation files (conditional logic, file refs, or agent delegation)
│   └── May link to reference files at specific steps
├── Reference files (DRY content shared by 2+ operations, leaves only)
├── Scripts (deterministic data extraction, reused 2+ times OR single-use with 20+ lines/piped commands)
└── Named agents (judgment work reused 2+ times)
```

Three principles:

1. **Deciding vs doing** — See Delegation > Behavior in the global CLAUDE.md. Operations must respect the deciding/doing boundary.
2. **Right-size the abstraction** — Inline if simple and self-contained. Extract to a file when complexity demands it. Extract to a script when data extraction is reused. Extract to an agent when judgment work is reused. Use the Skill tool for cross-skill workflows.
3. **References are DRY leaves** — They prevent update-in-N-places problems. Operations work without them for the happy path.

## Instructions

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

`name` and `description` are required. Optional fields (include only when needed):

- `argument-hint` — autocomplete hint shown after skill name (e.g., `[issue-number]`)
- `disable-model-invocation: true` — prevents Claude from auto-loading the skill; only the user can invoke it via `/`. Use for side-effect workflows where unintended invocation would be harmful.
- `user-invocable: false` — hides the skill from the `/` menu; only Claude can load it. Use for background knowledge that should augment Claude's behavior without user invocation.
- `allowed-tools` — comma-separated list of tools available when this skill is active (e.g., `Read, Grep, Glob`). Restricts Claude's available tools during the skill.
- `model` — model override for this skill (`sonnet`, `opus`, `haiku`)
- `context: fork` — runs the skill in a subagent (see Subagent Execution below)
- `agent` — subagent type when `context: fork` is set (`Explore`, `Plan`, `general-purpose`, or a custom agent type). Custom agent names reference files in `.claude/agents/<name>.md`.
- `hooks` — lifecycle hooks scoped to this skill
MUST NOT add keys not listed above.

### Description Rules

The `description` field is how Claude discovers and matches the skill to user intent. It MUST:

- **MUST use third person**: "Handles..." / "Creates..." / "Reviews..." -- not "Handle" or "I handle"
- **MUST state what AND when**: First clause says what it does, second says when to use it
- **MUST include trigger keywords**: Verbs and nouns a user would naturally say (e.g., "commit, push, rebase" for a git skill)
- **MUST be a single sentence**: One sentence, no line breaks, no bullet points
- **MUST stay under 1024 characters**: Long descriptions get truncated in skill listings
- **MUST NOT contain XML tags**

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

Custom agent types reference `.claude/agents/<name>.md` files; the name MUST match the filename without extension. Companion agents ship in `configs/claude/agents/` and are symlinked to `~/.claude/agents/` by `apply.sh`. The `skills` frontmatter field MAY be used in agent files to inject skill content into the subagent's system prompt before execution; list skill names as a YAML array (e.g., `skills: [git, compose]`). The skill's SKILL.md content is appended to the agent's prompt, giving it access to the skill's routing and references.

**`context: fork` vs Task tool**: Use `context: fork` when the entire skill runs in a subagent -- every invocation forks. Use the Task tool inside an operation step when only one step needs delegation and the orchestrator continues afterward. If every path through the operation delegates, prefer `context: fork` on SKILL.md.

### SKILL.md Body

The body after frontmatter is the hub that routes to operation files. Constraints:

- **Max 500 lines / 5000 tokens**: MUST NOT exceed these limits. Anything longer should be split into referenced files.
- **Heading structure**: MUST have one H1 (the skill title), then H2 for sections, H3 for individual operations
- **Required sections**: MUST have "Operations" (H2) listing each operation with a one-line summary and a link to its file
- **Optional sections**: MAY have "Combined Operations" (H2) for multi-operation intent mapping, "References" (H2) for shared reference files
- **Inline operations**: MAY contain inline operations for operations that meet ALL of these criteria: (1) linear sequence with no conditional branches, (2) no file references (patterns, guidelines, templates), (3) no agent delegation with context. MUST route to an operation file when any criterion is not met. A secondary test: does executing this operation require opening another file? If yes, it needs its own file. **Tiebreaker**: when an operation is structurally simple (inline-eligible) but consumes enough context to crowd out later work, context cost wins -- delegate to a subagent.

### Operation Files

Each operation file (e.g., `create-skill.md`, `review-skill.md`) contains the full instructions for one operation.

- **H1 heading**: MUST start with the operation name from SKILL.md. MAY append a descriptive word or phrase for clarity (e.g., `# Commit` or `# Apply Coding Preferences` for a `### Apply` entry). Do NOT use a generic "Operation" suffix — prefer standalone names or descriptive phrases.
- **Summary line**: MUST have one sentence after the heading describing what the operation does
- **Numbered steps**: MUST use numbered steps where each step has a **bold step name** followed by the instructions
- **Cross-references**: MUST reference files as plain text paths (e.g., references/commit-guidelines.md). Do NOT use markdown link syntax or backtick-wrapped paths.
- **Step nesting**: Steps MUST NOT nest sub-steps. If a step needs sub-steps, either flatten into sequential top-level steps or extract the sub-steps into a reference file.
- **Self-contained**: SHOULD be understandable from the operation file alone (referenced files provide detail, not essential context)

### Reference Files

Reference files DRY content shared by two or more operations that changes together.

- **Leaves only**: SKILL.md and operation files can reference these files. References MUST NOT reference other reference files. If a reference grows too large, split into sibling references that operations link to independently.
- **DRY threshold**: If content is only used by one operation, it belongs in the operation file, not a reference.
- **Descriptive names**: MUST describe the content type (e.g., `commit-guidelines.md`, `git-patterns.md`)
- **No operation logic**: MUST provide information, not step-by-step instructions
- **Size cap**: Reference files SHOULD stay under 300 lines. If a reference exceeds this, split into focused sibling references that operations link to independently.
- **Table of contents**: SHOULD include a table of contents for reference files over 100 lines
- **Inline linking**: When a step depends on reference content, link it at that step — not in a preamble or header
- **Operations stay executable**: Operations MUST be executable for the happy path without loading references

### Directory Structure

```
<skill-name>/
├── SKILL.md           # Hub (required)
├── <operation>.md     # One per operation (required, at least one)
├── references/        # Shared knowledge files (optional)
├── scripts/           # Executable scripts (optional)
├── agents/            # Companion agent files (optional)
└── assets/            # Non-text files (optional)
```

Reference files go in the `references/` subdirectory. Operation files stay top-level alongside SKILL.md.

### Skill Content Rules

- **MCP tool names**: SHOULD use fully qualified `ServerName:tool_name` format when referencing MCP tools
- **No cross-skill file references**: MUST NOT reference another skill's files via relative paths. Use the Skill tool for cross-skill delegation.
- **Patterns**: For Scripts vs Agents, Cross-skill Delegation, Named Agents, Interview, Deciding vs Doing, and Degrees of Freedom patterns, see content-patterns.md.
