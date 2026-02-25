# CLAUDE.md

## Architecture Summary

### Setup

- `apply.sh` is the main setup script: installs Homebrew/Nix, runs `brew bundle` on `Brewfile`, creates symlinks, builds Cargo tools
- Symlinks use `ensure_symlink()` and back up existing files to `.bak`

### Symlink Layout (configs/ -> target)

Full list of all symlinks is in README.md

### Cargo Tools

- `tools/md/` -- terminal markdown renderer (binary)
- `tools/gd/` -- terminal git diff viewer (binary)
- `tools/tui/` -- shared terminal UI library (lib crate, used by md and gd)
- These are a Cargo workspace under `tools/`. Each has its own `Cargo.toml`
- `tui` is a path dependency of both `md` and `gd`
- Binaries are built by `apply.sh` and symlinked to `~/.cargo/bin/`

### Fish Shell

- `configs/fish/conf.d/` -- auto-loaded config files; each MUST have `status is-interactive` guard

### Claude Code

- Agents are stateless -- each invocation evaluates fresh, no persistent memory

### Skills Locations

- **Global skills** (available everywhere): `~/Code/personal/dotfiles/configs/claude/skills/`
- **Gadget-specific skills** (only available in gadget projects): `~/Code/gadget/gadget/.claude/skills/`

### Agents

- `configs/claude/agents/` -- companion subagent files, symlinked to `~/.claude/agents/`
- Each agent owns a specific delegation domain (committing, PR writing, code review, CI triage, etc.)

### Commands

- `configs/claude/commands/` -- custom slash commands, symlinked to `~/.claude/commands/`

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.
