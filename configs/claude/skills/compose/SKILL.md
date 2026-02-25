---
name: compose
description: Creates, updates, reviews, and improves Claude Code skills, CLAUDE.md rules, and session prompts, decomposes large tasks into chunked execution plans, and writes handoff files for session continuity.
argument-hint: "[create | update | review | plan | handoff] [target]"
---

# Composing for Claude Code

Help create, update, review, and improve Claude Code skills, CLAUDE.md rules files, and session task prompts, and decompose large tasks into chunked execution plans.

## Operations

### Create Skill
Scaffold a new skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.
See [create-skill.md](create-skill.md) for detailed instructions.

### Update Skill
Add, modify, or remove operations, reference files, and SKILL.md content in an existing skill.
See [update-skill.md](update-skill.md) for detailed instructions.

### Review Skill
Evaluate an existing skill against best practices and report findings grouped by severity.
See [review-skill.md](review-skill.md) for detailed instructions.

### Create Rules
Write a CLAUDE.md or scoped rules file, producing clear instructions that configure Claude's behavior for a project.
See [create-rules.md](create-rules.md) for detailed instructions.

### Review Rules
Evaluate a CLAUDE.md or scoped rules file against best practices and report findings grouped by severity.
See [review-rules.md](review-rules.md) for detailed instructions.

### Create Prompt
Craft a session task prompt interactively, producing a polished prompt ready to paste into a new Claude Code session.
See [create-prompt.md](create-prompt.md) for detailed instructions.

### Review Prompt
Evaluate a session task prompt against best practices, report findings, and offer to improve it.
See [review-prompt.md](review-prompt.md) for detailed instructions.

### Create Handoff
Write a self-contained handoff and deliver via plan mode so the user can accept and continue in a fresh context.
See [create-handoff.md](create-handoff.md) for detailed instructions.

### Plan Task
Decompose a large task into ordered chunks with orchestrated subagent execution.
See [plan-task.md](plan-task.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"create and review"** / **"scaffold"** / **"new skill"** → Run Create Skill, then Review Skill on the new skill
- **"update skill"** / **"add operation"** / **"modify skill"** / **"change skill"** / **"add operation to"** / **"remove operation from"** / **"rename skill"** → Run Update Skill
- **"update and review"** → Run Update Skill, then Review Skill on the updated skill
- **"improve skill"** / **"fix skill"** → Run Review Skill, then apply the suggested fixes
- **"write CLAUDE.md"** / **"write rules"** / **"write instructions"** → Run Create Rules
- **"improve CLAUDE.md"** / **"review my instructions"** / **"fix my rules"** → Run Review Rules, then apply fixes
- **"write a prompt"** / **"craft a prompt"** / **"help me prompt"** / **"delegate this"** → Run Create Prompt
- **"review prompt"** / **"improve prompt"** / **"check my prompt"** → Run Review Prompt
- **"write and review prompt"** → Run Create Prompt, then Review Prompt on the result
- **"hand this off"** / **"handoff"** / **"save context"** / **"continue later"** / **"write what's left"** → Run Create Handoff
- **"plan this"** / **"break this down"** / **"chunk this"** / **"decompose this task"** → Run Plan Task
- **"review"** (ambiguous) → Present options via AskUserQuestion: "Review a skill", "Review a rules file", "Review a prompt"

## References

These files are referenced by the operation instructions. Operations that link to a reference file MUST read it before proceeding.

- [shared-rules.md](shared-rules.md) - Shared authoring rules (keyword conventions, content rules) for both skills and rules files
- [skill-spec.md](skill-spec.md) - Specification for authoring Claude Code skills (naming, frontmatter, structure, content rules)
- [rules-spec.md](rules-spec.md) - Specification for authoring CLAUDE.md and `.claude/rules/` rules files (locations, structure, content guidelines)
- [quality-checklist.md](quality-checklist.md) - Pass/fail evaluation criteria for skills and rules files
- [skill-template.md](skill-template.md) - Annotated templates for SKILL.md and operation files
- [content-patterns.md](content-patterns.md) - Reusable patterns for operation steps, task skills, and dynamic context injection
- [rules-template.md](rules-template.md) - Templates for CLAUDE.md and scoped rules files
- [prompt-spec.md](prompt-spec.md) - Prompt structure definition (Goal, Context, Requirements, Anti-requirements, Output) and style rules
- [plan-template.md](plan-template.md) - Templates for plan artifacts: master plan, chunk files, orchestrator prompt
