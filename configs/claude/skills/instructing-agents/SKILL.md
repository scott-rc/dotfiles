---
name: instructing-agents
description: Creates and reviews Claude Code agent skills and CLAUDE.md project instructions when users ask to scaffold, build, review, improve, or fix skills or write, update, or review CLAUDE.md rules files.
---

# Instructing Agents

Help create and review Claude Code agent skills and CLAUDE.md project instructions based on best practices.

## Operations

### Create Skill
Scaffold a new skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.
See [create-skill.md](create-skill.md) for detailed instructions.

### Review Skill
Evaluate an existing skill against best practices and report findings grouped by severity.
See [review-skill.md](review-skill.md) for detailed instructions.

### Create Rules
Write a CLAUDE.md or scoped rules file, producing clear instructions that configure Claude's behavior for a project.
See [create-rules.md](create-rules.md) for detailed instructions.

### Review Rules
Evaluate a CLAUDE.md or scoped rules file against best practices and report findings grouped by severity.
See [review-rules.md](review-rules.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"create and review"** / **"scaffold"** / **"new skill"** → Run Create Skill, then Review Skill on the new skill
- **"improve skill"** / **"fix skill"** → Run Review Skill, then apply the suggested fixes
- **"write CLAUDE.md"** / **"write rules"** / **"write instructions"** → Run Create Rules
- **"improve CLAUDE.md"** / **"review my instructions"** / **"fix my rules"** → Run Review Rules, then apply fixes
- **"review"** (ambiguous) → Ask the user whether they mean a skill or a rules file

**Important**: You MUST read and follow the detailed instruction file for each operation before executing it. Do not rely on the summaries above.

## References

These files are referenced by the operation instructions. Operations that link to a reference file MUST read it before proceeding.

- [spec.md](spec.md) - Authoring rules for skills (Skill Specification) and CLAUDE.md files (Rules Specification), plus shared content rules
- [quality-checklist.md](quality-checklist.md) - Pass/fail evaluation criteria for skills and rules files
- [skill-template.md](skill-template.md) - Annotated templates for SKILL.md and operation files
- [rules-template.md](rules-template.md) - Templates for CLAUDE.md and scoped rules files
