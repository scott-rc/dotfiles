# Dotfiles

macOS dotfiles managed via symlinks.

## Quick Start

```bash
./init.sh
```

This installs Homebrew, packages from `Brewfile`, Nix, and creates symlinks from config locations to this repo.

## Architecture

### init.sh

The main setup script that:

1. Installs Homebrew if missing
2. Installs all packages via `brew bundle`
3. Creates symlinks using `ensure_symlink()` (backs up existing files to `.bak`)
4. Configures iTerm2 preferences
5. Installs Nix if missing

### Brewfile

All Homebrew packages are declared here and installed eagerly during setup.

### configs/

Configuration directories symlinked to their expected locations:

| Directory | Target |
|-----------|--------|
| `claude/` | `~/.claude/` |
| `fish/` | `~/.config/fish/` |
| `ghostty/` | `~/Library/Application Support/com.mitchellh.ghostty/` |
| `git/` | `~/.gitconfig`, `~/.config/git/` |
| `karabiner/` | `~/.config/karabiner/` |
| `vim/` | `~/.vimrc`, `~/.ideavimrc`, `~/.config/nvim/` |
| `zed/` | `~/.config/zed/` |
| `zellij/` | `~/.config/zellij/` |

### Fish Shell Structure

```
configs/fish/
├── conf.d/          # Per-tool config (auto-loaded)
│   ├── git.fish     # Aliases and abbreviations
│   ├── node.fish    # npm/pnpm/yarn aliases
│   └── ...
├── functions/       # One function per file (auto-loaded on first call)
│   ├── gw.fish      # Git worktree switcher
│   ├── gwip.fish    # WIP commit
│   └── ...
└── fish_plugins     # Fisher plugin list
```

**conf.d files** contain:
- `status is-interactive` guard
- Aliases and abbreviations
- Tool initialization (e.g., `starship init fish | source`)

**functions/** contains one function per file, autoloaded by fish on first invocation.

## Useful Commands

| Command | Description |
|---------|-------------|
| `update_dotfiles` | Pull latest changes and re-run setup |
| `reload_dotfiles` | Restart fish shell |
| `edit_dotfiles` | Open repo in VS Code |
