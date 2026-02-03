# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is a macOS dotfiles repository that manages configuration files for various tools via symlinks. The repository is structured around a central `init.sh` script and a `configs/` directory containing all configuration files.

## Key Commands

- **Initialize/update dotfiles**: `./init.sh` - Creates symlinks from config locations to this repo. Existing files are backed up to `.bak`. Run with `LOG_LEVEL=debug ./init.sh` for verbose output.
- **Reload shell**: `reload` (fish function) - Restarts fish shell to apply changes

## Architecture

**init.sh**: The main setup script that:
- Installs Homebrew and Fish shell if missing
- Installs Nix package manager if missing
- Creates symlinks using `ensure_symlink()` which backs up existing files
- Configures iTerm2 preferences via `defaults write`
- Uses sudo for `/etc/*` paths (e.g., nix.conf)

**configs/**: Contains configuration directories symlinked to their expected locations:
- `atuin/` → `~/.config/atuin/config.toml`
- `claude/` → Individual files: `~/.claude/settings.json`, `~/.claude/commands`, `~/.claude/skills`, `~/.claude/hooks`
- `direnv/` → `~/.config/direnv/direnv.toml`
- `fish/` → `~/.config/fish` (primary shell, uses fisher plugin manager)
- `ghostty/` → `~/Library/Application Support/com.mitchellh.ghostty/config`
- `git/` → `~/.gitconfig`, `~/.config/git/.gitignore_global`
- `iterm2/` → Configured via `defaults write` (custom preferences folder)
- `karabiner/` → `~/.config/karabiner/karabiner.json`
- `nix/` → `/etc/nix/nix.conf`
- `orbstack/` → `~/.orbstack/config/docker.json`
- `starship/` → `~/.config/starship.toml`
- `vim/` → `~/.vimrc`, `~/.ideavimrc`, `~/.config/nvim/init.vim`
- `zed/` → `~/.config/zed/settings.json`, `~/.config/zed/keymap.json`
- `zellij/` → `~/.config/zellij/config.kdl`
- `zsh/` → `~/.zshrc`

**Fish shell structure** (`configs/fish/`):
- `completions/` - Custom completions for tools
- `conf.d/` - Per-tool configuration (loaded automatically by fish)
- `functions/` - Custom fish functions
- `fish_plugins` - Fisher plugin list (fisher, nvm.fish)
