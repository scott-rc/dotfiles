---
name: slides
description: Creates and manages Slidev markdown presentations, including scaffolding workspaces, authoring slide decks, and running dev/build/export workflows — use when the user mentions create presentations, write slides, Slidev, markdown slides, deck, talk, slide deck, or presentation.
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

1. **Locate talk**: discover `talks/` directories in the slides workspace. If multiple, present as AskUserQuestion options.
2. **Gather requirements**: determine what to write or change. If vague, batch clarifying questions into a single AskUserQuestion.
3. **Determine mode**: `create` if no `slides.md` or only starter template; `edit` if existing content.
4. **Delegate to `slide-writer` agent** with: talk_path, mode, requirements.
5. **Report**: relay slide count, file path, and build status.

### Present
Run dev server, build static output, or export to PDF/PNG.

1. **Identify talk**: if ambiguous, list available talks and present as AskUserQuestion options.
2. **Run by intent**:
   - **Dev server**: `cd <talk> && pnpm dev` -- serves at `http://localhost:3030`
   - **Build**: `cd <talk> && pnpm build` -- output to `dist/`
   - **Export PDF**: `cd <talk> && pnpm export` -- requires playwright
   - **Export PNG**: `cd <talk> && pnpm export -- --format png`
3. **Report**: server URL, output path, or output directory.

## Combined Operations

- **"new presentation"** / **"start a slide deck"** / **"scaffold slides"** → Run Setup then Create
- **"write slides"** / **"add slides"** / **"edit my deck"** → Run Author
- **"run my slides"** / **"preview presentation"** / **"build slides"** / **"export to PDF"** → Run Present
- **"new talk"** / **"add a talk"** → Run Create (assumes workspace exists)
- **"create presentation from scratch"** → Run Setup, Create, then Author

## References

These files are referenced by the operation instructions above:

- [slidev-syntax.md](slidev-syntax.md) — Slidev markdown syntax, layouts, animations, and advanced features
- [workspace-claude-template.md](workspace-claude-template.md) — CLAUDE.md template written into scaffolded workspaces
