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
   Read [workspace-claude-template.md](workspace-claude-template.md) and write its contents to `CLAUDE.md` in the workspace root.

4. **Initialize repo**:
   - `git init`
   - `pnpm install` — if this fails, verify pnpm is installed (`pnpm --version`) and the workspace root `package.json` is valid.
   - `git add .`
   - Create an initial commit with message `Initial slides workspace`
   - Report that the workspace is ready and suggest running the Create operation to add a talk
