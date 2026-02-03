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
3. Installs Fish shell if missing, adds to `/etc/shells`, sets as default
4. Installs Nix package manager if missing
5. Creates symlinks using `ensure_symlink()` (backs up existing files to `.bak`)
6. Configures iTerm2 preferences via `defaults write`
7. Disables ApplePressAndHoldEnabled for key repeat
8. Uses sudo for `/etc/*` paths (e.g., nix.conf)

### Brewfile

All Homebrew packages are declared here and installed eagerly during setup.

### configs/

Configuration directories symlinked to their expected locations:

| Directory | Target |
|-----------|--------|
| `atuin/` | `~/.config/atuin/config.toml` |
| `claude/` | `~/.claude/{settings.json,keybindings.json,commands,skills,hooks}` |
| `direnv/` | `~/.config/direnv/direnv.toml` |
| `fish/` | `~/.config/fish/` |
| `ghostty/` | `~/Library/Application Support/com.mitchellh.ghostty/config` |
| `git/` | `~/.gitconfig`, `~/.config/git/.gitignore_global` |
| `iterm2/` | via `defaults write` (custom preferences folder) |
| `karabiner/` | `~/.config/karabiner/karabiner.json` |
| `nix/` | `/etc/nix/nix.conf` (sudo) |
| `orbstack/` | `~/.orbstack/config/docker.json` |
| `starship/` | `~/.config/starship.toml` |
| `terminal/` | macOS Terminal color scheme |
| `vim/` | `~/.vimrc`, `~/.ideavimrc`, `~/.config/nvim/init.vim` |
| `zed/` | `~/.config/zed/{settings.json,keymap.json}` |
| `zellij/` | `~/.config/zellij/config.kdl` |
| `zsh/` | `~/.zshrc` |

### Fish Shell Structure

```
configs/fish/
├── completions/     # Custom completions for tools
├── conf.d/          # Per-tool config (auto-loaded)
│   ├── git.fish     # Aliases and abbreviations
│   ├── node.fish    # npm/pnpm/yarn aliases
│   └── ...
└── functions/       # One function per file (auto-loaded on first call)
    ├── gw.fish      # Git worktree switcher
    ├── gwip.fish    # WIP commit
    └── ...
```

**conf.d files** contain:
- `status is-interactive` guard
- Aliases and abbreviations
- Tool initialization (e.g., `starship init fish | source`)

**functions/** contains one function per file, autoloaded by fish on first invocation.

## Useful Commands

| Command | Description |
|---------|-------------|
| `./init.sh` | Run setup |
| `LOG_LEVEL=debug ./init.sh` | Verbose setup |
| `reload` | Restart fish shell |
