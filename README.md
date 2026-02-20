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
9. Builds CLI tools (md, gd) via Cargo
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
| `Cmd+Shift+*` | Ghostty | Repo/tab/window management |
| `Cmd+Ctrl+*` | Zellij | Worktree/pane management |

#### Ghostty (`Cmd+Shift`)

| Shortcut | Action |
|----------|--------|
| `Cmd+Shift+T` | New Ghostty tab (repo chooser) |
| `Cmd+Shift+W` | Close surface |
| `Cmd+Shift+N` | New Ghostty tab (repo chooser) |
| `Cmd+Shift+[/]` | Previous/next Ghostty tab |
| `Cmd+Shift+I/O` | Move Ghostty tab left/right |
| `Cmd+Shift+1-9` | Go to Ghostty tab by number |

#### Zellij (`Cmd+Ctrl`)

| Shortcut | Action |
|----------|--------|
| `Cmd+Ctrl+N` | New pane |
| `Cmd+Ctrl+W` | Close pane |
| `Cmd+Ctrl+H/J/K/L` | Focus left/down/up/right |
| `Cmd+Ctrl+F` | Toggle fullscreen |
| `Cmd+Ctrl+Z` | Toggle pane frames |
| `Cmd+Ctrl+\\` | Toggle floating panes |
| `Cmd+Ctrl+,/.` | Previous/next swap layout |
| `Cmd+Ctrl+[/]` | Previous/next Zellij tab |
| `Cmd+Ctrl+I/O` | Move tab left/right |
| `Cmd+Ctrl+X` | Close pane (alt) |
| `Cmd+Ctrl++/-/=` | Resize increase/decrease |
| `Cmd+Ctrl+Up/Down` | Focus up/down |
| `Cmd+Ctrl+G` | Lock mode |
| `Cmd+Ctrl+P` | Pane mode |
| `Cmd+Ctrl+T` | New Zellij tab |
| `Cmd+Ctrl+Y` | Tab mode |
| `Cmd+Ctrl+S` | Session mode |
| `Cmd+Ctrl+E` | Scroll mode |
| `Cmd+Ctrl+R` | Resize mode |
| `Cmd+Ctrl+M` | Move mode |

#### Neovim Git Diff

Global diff mode toggle with inline hunk preview, stage/unstage, and file navigation. Uses gitsigns + neo-tree git_status panel.

| Shortcut | Action |
|----------|--------|
| `Cmd+G` / `Ctrl+G` | Focus/toggle git changes panel (enables/disables diff highlights) |
| `Space gd` | Toggle diff mode (highlights + panel) |
| `Space gc` | Changed files vs base branch |
| `Space gw` | Working tree diff (unstaged changes) |
| `Space gi` | Staged changes (vs HEAD) |
| `Space gB` | Diff against branch (telescope picker) |
| `]c` / `[c` | Next/prev hunk |
| `]C` / `[C` | Last/first hunk |
| `]f` / `[f` | Next/prev changed file |
| `Space gs` | Stage hunk (visual: stage selected lines) |
| `Space gu` | Undo stage hunk |
| `Space gS` | Stage entire buffer |
| `Space gr` | Reset hunk (visual: reset selected lines) |
| `Space gR` | Reset entire buffer |
| `Space gp` | Preview hunk inline (shows deleted lines) |
| `Space gb` | Blame line |
| `ih` | Hunk text object (e.g., `dih`, `vih`, `yih`) |

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
| `md/` | Terminal markdown renderer (Rust) with color output, syntax highlighting (github-dark theme), YAML frontmatter support, word wrapping, pretty mode (default; Unicode box-drawing borders, bullets, decorations, hidden inline delimiters — disable with `--plain`), directory browsing (via `$SHELL` + `fzf`), and a built-in pager with terminal resize handling, style toggle, clipboard copy, and `$EDITOR` integration. Built by `apply.sh` and symlinked to `~/.cargo/bin/md`, so rebuilding with `cargo build --release` automatically updates the binary in PATH. |
| `gd/` | Terminal git diff viewer (Rust) with embedded neovim for syntax highlighting and full vim editing. Uses ratatui + crossterm for TUI, nvim `--embed` with `ext_linegrid` for rendering, and tree-sitter for syntax colors. Built by `apply.sh` and symlinked to `~/.cargo/bin/gd`. |

## Useful Commands

| Command | Description |
|---------|-------------|
| `./apply.sh` | Run setup |
| `LOG_LEVEL=debug ./apply.sh` | Verbose setup |
| `reload` | Restart fish shell |
