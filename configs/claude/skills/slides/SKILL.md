---
name: slides
description: Creates and manages Slidev markdown presentations -- use when the user mentions slides, slide deck, Slidev, or presentation.
---

# Slides

Help Claude create and manage Slidev markdown presentations from scaffolding through export.

## Operations

### Setup
Scaffold a new Slidev workspace with pnpm, shared dependencies, and directory structure.
See [setup.md](setup.md) for detailed instructions.

### Create
Create a new talk within an existing workspace, generating `slides.md` and `package.json`.
See [create.md](create.md) for detailed instructions.

### Author
Write and edit slide content using Slidev markdown syntax, layouts, and animations.
See [author.md](author.md) for detailed instructions.

### Present
Run dev server, build static output, or export to PDF/PNG.
See [present.md](present.md) for detailed instructions.

## Combined Operations

- **"new presentation"** / **"start a slide deck"** / **"scaffold slides"** → Run Setup then Create
- **"write slides"** / **"add slides"** / **"edit my deck"** → Run Author
- **"run my slides"** / **"preview presentation"** / **"build slides"** / **"export to PDF"** → Run Present
- **"new talk"** / **"add a talk"** → Run Create (assumes workspace exists)
- **"create presentation from scratch"** → Run Setup, Create, then Author

## References

- [slidev-syntax.md](references/slidev-syntax.md) — Slidev markdown syntax, layouts, animations, and advanced features
- [workspace-claude-template.md](references/workspace-claude-template.md) — CLAUDE.md template written into scaffolded workspaces
