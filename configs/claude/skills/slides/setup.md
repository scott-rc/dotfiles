# Setup

Scaffold the `~/Code/personal/slides/` pnpm workspace for Slidev presentations.

## Instructions

1. **Check for existing workspace**:
   If `~/Code/personal/slides/` already exists, report that the workspace is already set up and stop.

2. **Create workspace files**:
   - Create directory structure: `mkdir -p ~/Code/personal/slides/talks`
   - Write `pnpm-workspace.yaml`:
     ```yaml
     packages:
       - 'talks/*'

     catalog:
       '@slidev/cli': ^0
       '@slidev/theme-default': ^0
     ```
   - Write root `package.json`:
     ```json
     {
       "private": true,
       "name": "slides"
     }
     ```
   - Write `.gitignore`:
     ```
     node_modules
     dist
     .slidev
     ```

3. **Write CLAUDE.md**:
   Write `~/Code/personal/slides/CLAUDE.md` with the following template, adapted to match the workspace as scaffolded in step 2:
   ```markdown
   # Slides Workspace

   pnpm monorepo for Slidev presentations.

   ## Workspace Structure

   - `pnpm-workspace.yaml` defines workspace packages with `talks/*`
   - Each talk lives in `talks/<talk-name>/` with its own `package.json` and `slides.md`
   - Shared Slidev dependencies use the `catalog:` protocol in `pnpm-workspace.yaml`

   ## Creating a New Talk

   - MUST use kebab-case for talk directory names (e.g., `talks/intro-to-graphql/`)
   - Scaffold `talks/<name>/` with:
     - `slides.md` — slide content (headmatter + slides separated by `---`)
     - `package.json` — with `dev`, `build`, and `export` scripts pointing to `slidev`
   - `package.json` SHOULD reference shared dependencies via `catalog:` protocol
   - MAY copy an existing talk directory as a starting template

   ## Slide File Conventions

   - `slides.md` MUST start with headmatter (theme, title, transition, etc.)
   - Slides are separated by `---` on its own line
   - Presenter notes use `<!-- -->` HTML comments at the end of each slide
   - SHOULD keep one concept per slide
   - SHOULD use built-in layouts (`cover`, `two-cols`, `section`, etc.) before custom ones

   ## Common Commands

   Run all commands from the individual talk directory (`talks/<name>/`):

   - `pnpm dev` — start dev server with hot reload
   - `pnpm build` — build static SPA output
   - `pnpm export` — export slides to PDF (requires playwright)

   ## Dependencies

   - Shared Slidev packages are declared in `pnpm-workspace.yaml` under `catalog:`
   - Individual talks reference them with `catalog:` in their own `package.json`
   - Run `pnpm install` from the workspace root after adding new dependencies
   ```

4. **Initialize repo**:
   - `git init`
   - `pnpm install`
   - `git add .`
   - Create an initial commit with message `Initial slides workspace`
   - Report that the workspace is ready and suggest running the Create operation to add a talk
