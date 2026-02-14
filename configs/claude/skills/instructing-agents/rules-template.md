# Rules Templates

Annotated templates for creating CLAUDE.md and scoped rules files. Replace placeholders (`<...>`) with actual content.

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

### When to split into scoped rules

- Instructions apply to less than ~30% of the codebase
- The main CLAUDE.md exceeds ~200 lines
- Different directories have conflicting conventions (e.g., frontend vs backend)

### What to include vs exclude

| Include | Exclude |
|---------|---------|
| "Use bun, not npm" | "JavaScript uses `const` for constants" |
| "Run `make lint` before committing" | "Git commits should have messages" |
| "Tests go in `__tests__/` next to source" | "Write unit tests for your code" |
| "This monorepo has packages in `packages/`" | "A monorepo contains multiple packages" |
| "Use snake_case for database columns" | "SQL tables have columns" |
