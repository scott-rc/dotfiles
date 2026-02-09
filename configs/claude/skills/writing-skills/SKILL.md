---
name: writing-skills
description: Creates and reviews Claude Code agent skills when users ask to scaffold, build, review, improve, or fix skills following official best practices.
---

# Writing Skills

Help create and review Claude Code agent skills based on best practices.

## Operations

### Create
Scaffold a new skill interactively, producing a complete skill directory with SKILL.md, operation files, and reference files.
See [create.md](create.md) for detailed instructions.

### Review
Evaluate an existing skill against best practices and report findings grouped by severity.
See [review.md](review.md) for detailed instructions.

## Combined Operations

Users often request multiple operations together. Handle these as follows:

- **"create and review"** → Run create operation, then review operation on the new skill
- **"improve skill"** / **"fix skill"** → Run review operation, then apply the suggested fixes
- **"scaffold"** / **"new skill"** → Same as create operation

**Important**: For each operation, read and follow its detailed instruction file.

## References

These files are referenced by the operation instructions above:

- [spec.md](spec.md) - Naming, frontmatter, structure, and content rules for skills
- [quality-checklist.md](quality-checklist.md) - Pass/fail evaluation criteria for skill quality
- [skill-template.md](skill-template.md) - Annotated templates for SKILL.md and operation files
