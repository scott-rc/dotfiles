# Rules Templates

Annotated templates for creating CLAUDE.md and `.claude/rules/` files. Replace placeholders (`<...>`) with actual content.

## Project CLAUDE.md Template

```markdown
# CLAUDE.md

@README.md

---

<Section for build/test/lint commands the agent should run>

---

<Section for project-specific conventions Claude cannot infer>

---

<Section for architectural decisions and constraints>
```

**Guidance:**
- Start with `@README.md` so Claude has project context without duplicating it
- Use `---` separators between distinct instruction groups
- Each section should teach something Claude cannot figure out on its own

## Global CLAUDE.md Template

```markdown
# User Preferences

## <Category>

- <Specific preference or convention>
- <Another preference>

## Repository Map

<Map of repo names to paths, so Claude can resolve "check gadget" → ~/Code/gadget/gadget>
```

**Guidance:**
- Global instructions load in every conversation across all projects — keep them minimal
- Focus on personal workflow preferences, not project-specific details
- Repository maps help Claude navigate between projects by name

## CLAUDE.local.md Template

```markdown
# <Project-specific personal preferences>

<Instructions that apply to this project but shouldn't be checked in.>
<e.g., local dev URLs, personal test data, sandbox credentials>
```

**Guidance:**
- Auto-added to .gitignore — use for personal per-project preferences
- Same syntax as CLAUDE.md (supports `@file`, `---` separators, etc.)
- Loaded alongside project CLAUDE.md with the same priority

## User-level Rules Template

```markdown
# <Topic>

<Personal rules that apply across all projects. Place in ~/.claude/rules/<topic>.md>
```

**Guidance:**
- Loaded before project rules across all projects (project rules have higher priority)
- Use for personal workflow conventions (e.g., "always use bun", "never auto-commit")
- One concern per file, same as project-level unconditional rules

## Unconditional Rules Template

```markdown
# <Topic>

<Focused instructions for this topic. No frontmatter needed — loads every conversation.>
```

**Guidance:**
- Use to split a large CLAUDE.md into focused, topic-specific files
- Place in `.claude/rules/<topic>.md`
- Organize into subdirectories when there are many rules (e.g., `frontend/`, `backend/`)
- One concern per file (e.g., `code-style.md`, `testing.md`, `security.md`)

## Scoped Rules Template

```yaml
---
paths:
  - "<glob pattern>"
  - "<glob pattern>"
---
```

```markdown
<Focused instructions for files matching the paths above.>
```

**Guidance:**
- Use glob patterns: `src/api/**`, `*.test.ts`, `packages/core/**/*.ts`
- Brace expansion supported: `src/**/*.{ts,tsx}`, `{src,lib}/**/*.ts`
- Keep scoped rules short — they load alongside the main CLAUDE.md
- One concern per file (e.g., API conventions, test patterns, component guidelines)

## Content Guidance

### When to use @file

- The source file is maintained independently and may change (README, CONTRIBUTING, API docs)
- The content is long and would bloat CLAUDE.md
- Multiple CLAUDE.md files need the same information

### When NOT to use @file

- The instruction is a one-liner (just write it directly)
- The referenced file contains mostly irrelevant content (extract the relevant parts instead)

### When to split into `.claude/rules/` files

- The main CLAUDE.md exceeds ~200 lines — split topic-specific sections into unconditional rules files
- Instructions apply to less than ~30% of the codebase — use scoped rules with `paths:` frontmatter
- Different directories have conflicting conventions (e.g., frontend vs backend) — use scoped rules or subdirectories

### What to include vs exclude

**Include** (Claude cannot infer these):
- "Use bun, not npm"
- "Run `make lint` before committing"
- "Tests go in `__tests__/` next to source"
- "This monorepo has packages in `packages/`"
- "Use snake_case for database columns"

**Exclude** (common knowledge, adds no value):
- "JavaScript uses `const` for constants"
- "Git commits should have messages"
- "Write unit tests for your code"
- "A monorepo contains multiple packages"
- "SQL tables have columns"

### The conciseness test

For each line, ask: "Would removing this cause Claude to make mistakes?" If not, cut it.

### When Claude ignores rules

The file is probably too long and the rule is getting lost. Prune aggressively rather than adding more emphasis or more rules. Shorter files with fewer, stronger rules outperform long files with comprehensive coverage.

### Emphasis for critical rules

Use "IMPORTANT" or "YOU MUST" to improve adherence on critical instructions. Use sparingly — if everything is emphasized, nothing is.
