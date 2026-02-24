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
   MUST read [slides-claude-md.md](slides-claude-md.md) and copy the template content into `~/Code/personal/slides/CLAUDE.md`. Adapt the template to match the workspace as scaffolded in step 2.

4. **Initialize repo**:
   - `git init`
   - `pnpm install`
   - `git add .`
   - Create an initial commit with message `Initial slides workspace`
   - Report that the workspace is ready and suggest running the Create operation to add a talk
