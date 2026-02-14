# Dotfiles

macOS dotfiles managed via symlinks.

## Quick Start

```bash
./apply.sh
```

This installs Homebrew, packages from `Brewfile`, Nix, and creates symlinks from config locations to this repo.

## Architecture

### apply.sh

The main setup script that:

1. Sets macOS defaults (disables ApplePressAndHoldEnabled, remaps Minimize shortcut)
2. Installs Homebrew if missing
3. Installs all packages via `brew bundle`
4. Creates symlinks using `ensure_symlink()` (backs up existing files to `.bak`)
5. Adds Fish shell to `/etc/shells`
6. Configures iTerm2 preferences via `defaults write`
7. Installs Nix package manager if missing
8. Installs CLI tools (md) via Deno
9. Uses sudo for `/etc/*` paths (e.g., nix.conf)

### Brewfile

All Homebrew packages are declared here and installed eagerly during setup.

### configs/

Configuration directories symlinked to their expected locations:

| Directory | Target |
|-----------|--------|
| `atuin/` | `~/.config/atuin/config.toml` |
| `bat/` | `~/.config/bat` |
| `claude/` | `~/.claude/{CLAUDE.md,settings.json,keybindings.json,commands,skills,hooks,statusline}` |
| `direnv/` | `~/.config/direnv/direnv.toml` |
| `fish/` | `~/.config/fish/` |
| `ghostty/` | `~/Library/Application Support/com.mitchellh.ghostty/config` |
| `git/` | `~/.gitconfig`, `~/.config/git/.gitignore_global` |
| `gitui/` | `~/.config/gitui/theme.ron` |
| `glow/` | `~/Library/Preferences/glow/glow.yml` |
| `iterm2/` | via `defaults write` (custom preferences folder) |
| `karabiner/` | `~/.config/karabiner/karabiner.json` |
| `lazygit/` | `~/Library/Application Support/lazygit/config.yml` |
| `nix/` | `/etc/nix/nix.conf` (sudo) |
| `orbstack/` | `~/.orbstack/config/docker.json` |
| `starship/` | `~/.config/starship.toml` |
| `vim/` | `~/.config/nvim/init.lua` |
| `zed/` | `~/.config/zed/{settings.json,keymap.json}` |
| `zellij/` | `~/.config/zellij/{config.kdl,layouts}` |
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
    ├── gsw.fish     # Git switch helper
    ├── gwip.fish    # WIP commit
    └── ...
```

**conf.d files** contain:
- `status is-interactive` guard
- Aliases and abbreviations
- Tool initialization (e.g., `starship init fish | source`)

**functions/** contains one function per file, autoloaded by fish on first invocation.

### tools/

Custom CLI tools built with Deno.

| Tool | Description |
|------|-------------|
| `md/` | Terminal markdown renderer with color output, syntax highlighting, YAML frontmatter support, word wrapping, directory browsing (via `$SHELL` + `fzf`), and a built-in pager with terminal resize handling, clipboard copy, and `$EDITOR` integration. Install globally via `deno task install`. |

## Useful Commands

| Command | Description |
|---------|-------------|
| `./apply.sh` | Run setup |
| `LOG_LEVEL=debug ./apply.sh` | Verbose setup |
| `reload` | Restart fish shell |
