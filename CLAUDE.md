# CLAUDE.md

## Architecture Summary

### Setup

- `apply.sh` is the main setup script: installs Homebrew/Nix, runs `brew bundle` on `Brewfile`, creates symlinks, builds Cargo tools
- Symlinks use `ensure_symlink()` and back up existing files to `.bak`

### Cargo Tools

- `tools/tui/` is a workspace dependency of both `tools/md/` and `tools/gd/`

### Fish Shell

- `configs/fish/conf.d/` -- auto-loaded config files; each MUST have `status is-interactive` guard

## Claude Code

- Skills: `configs/claude/skills/`
- Agents: `configs/claude/agents/`
- Commands: `configs/claude/commands/`

## Build & Test

- `./apply.sh` -- full setup (Homebrew, Nix, symlinks, Cargo tools)
- `cargo build --release` in `tools/` (workspace root) -- build all tools
- `cargo test` in `tools/<name>/` -- per-tool tests

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.
