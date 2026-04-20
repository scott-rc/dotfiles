# Claude Code Configuration

This directory contains configuration files for [Claude Code](https://claude.ai/code).

## Tracked Files

- `settings.json` - Main Claude Code settings (permissions, plugins, etc.)
- `keybindings.json` - Key binding overrides
- `CLAUDE.md` - Global preferences loaded by every session
- `skills/` - Custom skill definitions (multi-file commands)
- `rules/` - Scoped rules (path- or context-scoped guidance)
- `hooks/` - Hook scripts invoked by settings.json
- `statusline` - Status line script

## Untracked Files

The following files/directories remain in `~/.claude/` but are NOT tracked in version control:

- `cache/` - Downloaded cache files
- `debug/` - Debug logs
- `file-history/` - File edit history
- `history.jsonl` - Command history
- `ide/` - IDE-specific temporary files
- `plans/` - Generated plans
- `plugins/cache/` - Plugin cache
- `session-env/` - Session environment snapshots
- `stats-cache.json` - Usage statistics
- `telemetry/` - Telemetry data
- `todos/` - Todo list storage

## Setup

The repo-root `apply.sh` creates these symlinks:
- `~/.claude/CLAUDE.md` → `configs/claude/CLAUDE.md`
- `~/.claude/settings.json` → `configs/claude/settings.json`
- `~/.claude/keybindings.json` → `configs/claude/keybindings.json`
- `~/.claude/skills/` → `configs/claude/skills/`
- `~/.claude/rules/` → `configs/claude/rules/`
- `~/.claude/hooks/` → `configs/claude/hooks/`
- `~/.claude/statusline` → `configs/claude/statusline`

## Shared With Codex

For Codex/Agents symlink mappings, see the repo-root `README.md` under `Shared Configuration (Claude Authority)`.
