# Rules Specification

Rules for authoring CLAUDE.md and `.claude/rules/` rules files. All operations in this skill validate against these rules.

These rules supplement the shared authoring rules.

## Overview

CLAUDE.md files provide persistent instructions that Claude loads into every conversation. They configure Claude's behavior for a project or globally. Rules files (`.claude/rules/*.md`) provide modular, topic-specific project instructions — either unconditional or scoped to specific file paths.

## File Locations

- `CLAUDE.md` or `.claude/CLAUDE.md` (project root) — Project-wide, loads every conversation in that project
- `CLAUDE.md` (subdirectory) — Subtree, loads when working on files in that subtree
- `CLAUDE.local.md` (project root) — Private per-project instructions, auto-added to .gitignore. Use for personal preferences that shouldn't be checked in (local dev URLs, personal test data, sandbox credentials).
- `~/.claude/CLAUDE.md` — Global (user), loads every conversation across all projects
- `.claude/rules/*.md` (no `paths:` frontmatter) — Unconditional project rules, always loaded with the same priority as `.claude/CLAUDE.md`
- `.claude/rules/*.md` (with `paths:` frontmatter) — Scoped rules, loads only when matching files are active
- `~/.claude/rules/*.md` — User-level rules, loaded before project rules across all projects
- Managed policy (macOS: `/Library/Application Support/ClaudeCode/CLAUDE.md`) — Organization-wide policy, requires IT/DevOps deployment

CLAUDE.md files cascade: global, then project root, then subdirectories. More specific files supplement, not override, broader ones. User-level rules load before project rules, giving project rules higher priority.

## Structure

- **Headings**: SHOULD use H1 for the file title, H2 for major sections. Keep the heading hierarchy flat.
- **@file references**: Use `@filename` to include content from other files (e.g., `@README.md`). Claude reads the referenced file as additional context. Place these near the top of the file.
- **Section separators**: Use `---` (horizontal rule) to visually separate distinct instruction groups.
- **Brevity**: MUST be concise. Every line loads into every conversation — verbosity has a direct token cost.

## Content Guidelines

These guidelines supplement the shared authoring rules in shared-rules.md.

**What to include:**
- Project-specific conventions Claude cannot infer (e.g., "use bun, not npm", "tests go in `__tests__/`")
- Build, test, and lint commands (e.g., "run `make check` before committing")
- Architectural decisions and constraints (e.g., "this is a monorepo with packages in `packages/`")
- File organization patterns (e.g., "one component per file, co-locate tests")
- References to key documentation via `@file` (e.g., `@README.md`, `@CONTRIBUTING.md`)

**What NOT to include:**
- Information already in referenced files (don't duplicate `@README.md` content)

### Conciseness Test

For each line, ask: "Would removing this cause Claude to make mistakes?" If not, cut it. If Claude keeps ignoring a rule, the file is probably too long — prune aggressively rather than adding more rules. Treat CLAUDE.md like code: review when things go wrong, prune regularly.

### Emphasis

You can tune instruction adherence by adding emphasis ("IMPORTANT", "YOU MUST"). Use sparingly — if everything is important, nothing is.

## @file References

`@filename` tells Claude to read another file as additional context. Use it to:

- Pull in README, CONTRIBUTING, or architecture docs without duplicating them
- Reference style guides, API schemas, or other living documents

Syntax: `@path/to/file` on its own line or inline. Both relative and absolute paths are allowed; relative paths resolve from the file's location. Imported files can recursively import other files (max depth 5 hops). Imports are not evaluated inside markdown code spans or code blocks.

SHOULD prefer `@file` over copying content. If the source file changes, the reference stays current.

## Modular Rules (`.claude/rules/`)

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

## Anti-patterns

- **Duplicating README content**: Use `@README.md` instead of copying setup instructions
- **Common knowledge**: Don't teach Claude things it already knows (e.g., "use `git add` before `git commit`")
- **Vague instructions**: "Follow best practices" is not actionable. State the specific practice.
- **Excessive length**: If CLAUDE.md exceeds ~200 lines, split into `.claude/rules/` files or use `@file` references
- **Unstable references**: Don't hardcode version numbers, specific dates, or URLs that may change
- **Over-specified files**: If CLAUDE.md is so long that Claude ignores rules, it needs aggressive pruning, not more rules. Adding emphasis to every instruction is a symptom of this problem.
- **Kitchen-sink context**: Adding unrelated instructions to a single file reduces effectiveness. Split by topic or scope instead.
