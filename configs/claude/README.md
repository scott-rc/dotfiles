# Claude Code Configuration

This directory contains configuration files for [Claude Code](https://claude.ai/code).

## Tracked Files

- `settings.json` - Main Claude Code settings (permissions, plugins, etc.)
- `commands/*.md` - Custom command definitions
- `skills/` - Custom skill definitions (multi-file commands)

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

The `init.sh` script creates symlinks:
- `~/.claude/settings.json` → `configs/claude/settings.json`
- `~/.claude/commands/` → `configs/claude/commands/`
- `~/.claude/skills/` → `configs/claude/skills/`

## Shared With Codex

For Codex/Agents symlink mappings, see `README.md` under `Shared Agent Configuration (Claude Authority)`.
