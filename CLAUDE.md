# CLAUDE.md

## Architecture Summary

### Setup

- `apply.sh` is the main setup script: installs Homebrew/Nix, runs `brew bundle` on `Brewfile`, creates symlinks, builds Cargo tools
- Symlinks use `ensure_symlink()` and back up existing files to `.bak`

### Symlink Layout (configs/ -> target)

- `configs/fish/` -> `~/.config/fish/`
- `configs/vim/` -> `~/.config/nvim/init.lua`
- `configs/git/` -> `~/.gitconfig`, `~/.config/git/.gitignore_global`
- `configs/claude/` -> `~/.claude/{CLAUDE.md,settings.json,keybindings.json,commands,skills,hooks,statusline,rules}`
- `configs/claude/agents/` -> `~/.claude/agents/`
- `configs/zed/` -> `~/.config/zed/{settings.json,keymap.json}`
- `configs/ghostty/` -> `~/Library/Application Support/com.mitchellh.ghostty/config`
- `configs/zellij/` -> `~/.config/zellij/{config.kdl,layouts}`
- Full list of all symlinks is in README.md

### Cargo Tools

- `tools/md/` -- terminal markdown renderer (binary)
- `tools/gd/` -- terminal git diff viewer (binary)
- `tools/tui/` -- shared terminal UI library (lib crate, used by md and gd)
- These are a Cargo workspace under `tools/`. Each has its own `Cargo.toml`
- `tui` is a path dependency of both `md` and `gd`
- Binaries are built by `apply.sh` and symlinked to `~/.cargo/bin/`

### Fish Shell

- `configs/fish/conf.d/` -- auto-loaded config files; each MUST have `status is-interactive` guard
- `configs/fish/functions/` -- one function per file, autoloaded by fish on first call
- `configs/fish/completions/` -- custom completions

### Claude Code

- Skills: `configs/claude/skills/` (symlinked to `~/.claude/skills/`)
- Agents: `configs/claude/agents/` (symlinked to `~/.claude/agents/`)
- Agents are stateless -- each invocation evaluates fresh, no persistent memory

---

When making changes to the codebase, keep README.md in sync. Any new features, changed options, added keybindings, or modified architecture MUST be reflected in the README.

---

After changing a tool in `tools/`, MUST run `cargo build --release` in the tool's directory. The release binary is symlinked into PATH by `apply.sh`, so a debug-only build leaves the installed binary stale. Changes to `tools/tui/` require rebuilding both `tools/md/` and `tools/gd/`, since they depend on tui as a workspace dependency.