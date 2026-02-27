---
name: compose
description: Creates, updates, reviews, and improves Claude Code skills, CLAUDE.md rules, and session prompts, decomposes large tasks into chunked execution plans, and writes handoff files for session continuity.
argument-hint: "[create | update | review | plan | handoff] [target]"
---

# Composing for Claude Code

## Operations

### Create Skill
Scaffold a new skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.
See operations/create-skill.md for detailed instructions.

### Update Skill
Add, modify, or remove operations, reference files, and SKILL.md content in an existing skill.
See operations/update-skill.md for detailed instructions.

### Review Skill
Evaluate an existing skill against best practices and report findings grouped by severity.
See operations/review-skill.md for detailed instructions.

### Create Rules
Write a CLAUDE.md or scoped rules file, producing clear instructions that configure Claude's behavior for a project.
See operations/create-rules.md for detailed instructions.

### Review Rules
Evaluate a CLAUDE.md or scoped rules file against best practices and report findings grouped by severity.
See operations/review-rules.md for detailed instructions.

### Create Prompt
Craft a session task prompt interactively, producing a polished prompt ready to paste into a new Claude Code session.
See operations/create-prompt.md for detailed instructions.

### Review Prompt
Evaluate a session task prompt against best practices, report findings, and offer to improve it.
See operations/review-prompt.md for detailed instructions.

### Create Handoff
Write a self-contained handoff and deliver via plan mode so the user can accept and continue in a fresh context.
See operations/create-handoff.md for detailed instructions.

### Plan Task
Decompose a large task into ordered chunks with orchestrated subagent execution.
See operations/plan-task.md for detailed instructions.

## Delegation

Within compose, all writing MUST go through `skill-writer` or `rules-writer`. MUST NOT delegate to `code-writer` — it lacks skill/rules validation and its code-oriented workflows (TDD, build, lint) are meaningless for markdown skill files.

- Skill files (operations, references, SKILL.md) — `skill-writer` (supports both create and update modes)
- Rules files (CLAUDE.md, `.claude/rules/`) — `rules-writer`
- Review-fix cycles — same agents: `skill-writer` for skill fixes, `rules-writer` for rules fixes

Full loop mechanics (cycle count, pass criteria, disagreement handling): references/multi-perspective-review.md

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"create and review"** / **"scaffold"** / **"new skill"** → Run Create Skill, then Review Skill on the new skill
- **"update skill"** / **"add operation"** / **"modify skill"** / **"change skill"** / **"add operation to"** / **"remove operation from"** / **"rename skill"** → Run Update Skill
- **"update and review"** → Run Update Skill (includes review loop)
- **"improve skill"** / **"fix skill"** → Run Review Skill, apply fixes via the review-fix loop until all 3 agents pass
- **"write CLAUDE.md"** / **"write rules"** / **"write instructions"** → Run Create Rules
- **"improve CLAUDE.md"** / **"review my instructions"** / **"fix my rules"** → Run Review Rules, apply fixes via the review-fix loop until all 3 agents pass
- **"write a prompt"** / **"craft a prompt"** / **"help me prompt"** / **"delegate this"** → Run Create Prompt
- **"review prompt"** / **"improve prompt"** / **"check my prompt"** → Run Review Prompt
- **"write and review prompt"** → Run Create Prompt, then Review Prompt on the result
- **"hand this off"** / **"handoff"** / **"save context"** / **"continue later"** / **"write what's left"** → Run Create Handoff
- **"plan this"** / **"break this down"** / **"chunk this"** / **"decompose this task"** → Run Plan Task
- **"review"** (ambiguous) → Present options via AskUserQuestion: "Review a skill", "Review a rules file", "Review a prompt"

## References

- references/shared-rules.md - Shared authoring rules (keyword conventions, content rules) for both skills and rules files
- references/skill-spec.md - Specification for authoring Claude Code skills (naming, frontmatter, structure, content rules)
- references/rules-spec.md - Specification for authoring CLAUDE.md and `.claude/rules/` rules files (locations, structure, content guidelines)
- references/quality-checklist.md - Pass/fail evaluation criteria for skills and rules files
- references/skill-template.md - Annotated templates for SKILL.md and operation files
- references/content-patterns.md - Reusable patterns for operation steps, task skills, and dynamic context injection
- references/rules-template.md - Templates for CLAUDE.md and scoped rules files
- references/multi-perspective-review.md - Three-agent parallel review loop (Sonnet/Opus/Haiku) with convergence criteria
- references/plan-template.md - Templates for plan artifacts: master plan, chunk files, orchestrator prompt
