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
8. Sets up Rust toolchain and wasm32-wasip1 target via rustup
9. Builds CLI tools (md) via Cargo
10. Uses sudo for `/etc/*` paths (e.g., nix.conf)

### Brewfile

All Homebrew packages are declared here and installed eagerly during setup.

### configs/

Configuration directories symlinked to their expected locations:

| Directory | Target |
|-----------|--------|
| `atuin/` | `~/.config/atuin/config.toml` |
| `bat/` | `~/.config/bat` |
| `claude/` | `~/.claude/{CLAUDE.md,settings.json,keybindings.json,commands,skills,hooks,statusline,rules}` |
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
    ├── zrepo.fish   # Zellij repo session chooser
    └── ...
```

**conf.d files** contain:
- `status is-interactive` guard
- Aliases and abbreviations
- Tool initialization (e.g., `starship init fish | source`)

**functions/** contains one function per file, autoloaded by fish on first invocation.

### Keybinding Scheme

Three modifier layers to avoid conflicts:

| Layer | Modifier | Purpose |
|-------|----------|---------|
| `Cmd+*` | unbound from Ghostty | Pass through to CLI tools (Neovim, etc.) |
| `Cmd+Ctrl+*` | Ghostty | Repo/tab/window management |
| `Cmd+Shift+*` | Zellij | Worktree/pane management |

#### Ghostty (`Cmd+Ctrl`)

| Shortcut | Action |
|----------|--------|
| `Cmd+Ctrl+T` | New Ghostty tab (repo chooser) |
| `Cmd+Ctrl+W` | Close surface |
| `Cmd+Ctrl+N` | New Ghostty tab (repo chooser) |
| `Cmd+Ctrl+[/]` | Previous/next Ghostty tab |
| `Cmd+Ctrl+1-9` | Go to Ghostty tab by number |

#### Zellij (`Cmd+Shift`)

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+N` | New pane |
| `Cmd+Shift+W` | Close pane |
| `Cmd+Shift+H/J/K/L` | Focus left/down/up/right |
| `Cmd+Shift+F` | Toggle fullscreen |
| `Cmd+Shift+Z` | Toggle pane frames |
| `Cmd+Shift+\\` | Toggle floating panes |
| `Cmd+Shift+,/.` | Previous/next swap layout |
| `Cmd+Shift+[/]` | Previous/next Zellij tab |
| `Cmd+Shift+I/O` | Move tab left/right |
| `Cmd+Shift+X` | Close pane (alt) |
| `Cmd+Shift++/-/=` | Resize increase/decrease |
| `Cmd+Shift+Up/Down` | Focus up/down |
| `Cmd+Shift+G` | Lock mode |
| `Cmd+Shift+P` | Pane mode |
| `Cmd+Shift+T` | Tab mode (then `n` for new tab) |
| `Cmd+Shift+S` | Session mode |
| `Cmd+Shift+E` | Scroll mode |
| `Cmd+Shift+R` | Resize mode |
| `Cmd+Shift+M` | Move mode |

### Workflow: Ghostty Tabs + Zellij Sessions

Each Ghostty tab runs its own Zellij session for a repo. Zellij tabs represent worktrees within a repo, and panes are free-form within each worktree tab.

```
Ghostty window (tab bar: [gadget] [dotfiles] [ggt])
├── Ghostty tab: gadget    → Zellij session "gadget"
│   ├── Zellij tab: main
│   ├── Zellij tab: fix-login
│   └── Zellij tab: add-auth
├── Ghostty tab: dotfiles  → Zellij session "dotfiles"
│   └── Zellij tab: main
└── Ghostty tab: ggt       → Zellij session "ggt"
    └── Zellij tab: main
```

The `zrepo` fish function handles session selection — dynamically discovers repos under `~/Code/*/*` (any directory containing `.git`), presents an fzf picker with git preview (branch, recent commits, status) when called with no args, or attaches directly with `zrepo <name>` (matches `category/name` or just `name`). Ghostty's `command` is set to `zrepo` so each new tab gets the picker. Zellij tabs auto-rename to the current git branch via a `fish_prompt` hook.

### tools/

Custom CLI tools.

| Tool | Description |
|------|-------------|
| `md/` | Terminal markdown renderer (Rust) with color output, syntax highlighting (github-dark theme), YAML frontmatter support, word wrapping, directory browsing (via `$SHELL` + `fzf`), and a built-in pager with terminal resize handling, clipboard copy, and `$EDITOR` integration. Built by `apply.sh` and symlinked to `~/.cargo/bin/md`, so rebuilding with `cargo build --release` automatically updates the binary in PATH. |

## Useful Commands

| Command | Description |
|---------|-------------|
| `./apply.sh` | Run setup |
| `LOG_LEVEL=debug ./apply.sh` | Verbose setup |
| `reload` | Restart fish shell |
