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
9. Builds CLI tools via Cargo workspace (md, gd, tui)
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
| `codex/` | `~/.codex/config.toml`, `~/.codex/rules/default.rules` |
| `configs/claude/CLAUDE.md` | `~/.codex/AGENTS.md` (Codex global AGENTS guidance) |
| `cursor/` | `~/Library/Application Support/Cursor/User/{settings.json,keybindings.json}`, `~/.cursor/mcp.json`, extensions via `cursor --install-extension` |
| `direnv/` | `~/.config/direnv/direnv.toml` |
| `fish/` | `~/.config/fish/` |
| `ghostty/` | `~/Library/Application Support/com.mitchellh.ghostty/config` |
| `git/` | `~/.gitconfig`, `~/.config/git/.gitignore_global` |
| `gitui/` | `~/.config/gitui/theme.ron` |
| `glow/` | `~/Library/Preferences/glow/glow.yml` |
| `iterm2/` | via `defaults write` (custom preferences folder) |
| `karabiner/` | `~/.config/karabiner/karabiner.json` |
| `lazygit/` | `~/Library/Application Support/lazygit/config.yml` |
| `lsd/` | `~/.config/lsd` |
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
    ├── gwc.fish     # Clean merged/orphaned worktrees
    ├── gwip.fish    # WIP commit
    ├── gwt.fish     # Create git worktree for a task
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
| `Cmd+Ctrl+I/O` | Move Ghostty tab left/right |
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
| `Cmd+Shift+T` | New Zellij tab |
| `Cmd+Shift+Y` | Tab mode |
| `Cmd+Shift+S` | Session mode |
| `Cmd+Shift+U` | Enter scroll mode + half-page up |
| `Cmd+Shift+D` | Enter scroll mode + half-page down |
| `Cmd+Shift+R` | Resize mode |
| `Cmd+Shift+M` | Move mode |

#### Neovim Git Diff

Global diff mode toggle with inline hunk preview, stage/unstage, and file navigation. Uses gitsigns + neo-tree git_status panel. Features read-only mode for historical diffs, active `.git/index` watching for real-time refresh, and a CLI entry point (`vd`).

| Shortcut | Action |
|----------|--------|
| `Cmd+G` / `Ctrl+G` | Focus/toggle git changes panel (enables/disables diff highlights) |
| `Space gd` | Toggle diff mode (highlights + panel) |
| `Space gc` | Changed files vs base branch (read-only) |
| `Space gw` | Working tree diff (unstaged changes) |
| `Space gi` | Staged changes (vs HEAD) |
| `Space gB` | Diff against branch (telescope picker, read-only) |
| `Space gD` | Diff source picker (branches, commits, working tree, staged) |
| `Space gX` | Diff against arbitrary ref (freeform input) |
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
| `Space yh` | Copy hunk to clipboard (visual: copy selection) |
| `ih` | Hunk text object (e.g., `dih`, `vih`, `yih`) |

#### Zed Git Diff

Git panel toggle with hunk-level staging and vim-style leader bindings. Mirrors the Neovim workflow where possible.

| Shortcut | Context | Action |
|----------|---------|--------|
| `Cmd+G` | Global | Toggle git panel (right dock) |
| `Cmd+Shift+G` | Global | Toggle inline/split diff view |
| `Cmd+1` | Global | Focus git panel changes list |
| `Cmd+1` | Git panel | Focus back to editor |
| `Space gd` | Editor (normal) | Working tree diff |
| `Space gc` | Editor (normal) | Branch diff (vs default branch) |
| `Space gs` | Editor (normal) | Stage file |
| `Space gu` | Editor (normal) | Unstage file |
| `Space gr` | Editor (normal) | Restore hunk |
| `Space gR` | Editor (normal) | Restore file |
| `Space gh` | Editor (normal) | File history |
| `Space gp` | Editor (normal) | Toggle inline diff hunks |
| `]c` / `[c` | Editor (normal) | Next/prev hunk (built-in vim) |
| `d o` | Editor (normal) | Expand diff hunk inline (built-in vim) |
| `d u` / `d U` | Editor (normal) | Stage hunk / stage all (built-in vim) |
| `d p` | Editor (normal) | Restore hunk (built-in vim) |
| `space` | Git panel | Open/navigate to file |
| `a` | Git panel | Toggle staged |
| `d` | Git panel | Discard changes |
| `t` | Git panel | Toggle tree/list view |

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
| `md/` | Terminal markdown renderer (Rust) with color output, syntax highlighting (github-dark theme), YAML frontmatter support, word wrapping, pretty mode (default; Unicode box-drawing borders, bullets, decorations, hidden inline delimiters, links show only text with blue underline and OSC 8 clickable hyperlinks — disable with `--plain`), directory browsing (via `$SHELL` + `fzf`), and a built-in pager with terminal resize handling, style toggle, clipboard copy, `$EDITOR` integration, link navigation (Tab/Shift-Tab to cycle links with background-highlighted line, Enter to follow `.md` file links stacked within the pager, Backspace to go back), and mouse support (click links to follow, scroll wheel). Includes criterion microbenchmarks (`cargo bench`) for the render pipeline, wrapping, and syntax highlighting. Built by `apply.sh` and symlinked to `~/.cargo/bin/md`, so rebuilding with `cargo build --release` automatically updates the binary in PATH. |
| `gd/` | Terminal git diff viewer (Rust) with syntax highlighting (syntect, GitHub Dark theme), word-level diff highlights, dual line numbers, `↪` wrap continuation markers, and a built-in pager (modularized under `src/pager/`) with hunk/file navigation with centered viewport (`d`/`u`/`D`/`U`, change-group navigation in full-context mode, `d`/`u` always global across files, file position status), content-line cursor skipping (skips headers/separators, lands only on diff content), full-file context toggle with scrollbar (`z`), UTF-8-safe search cursor editing, `$EDITOR` delegation (`E`, with repo-root path resolution so subdirectory launches open the right file), a toggleable file tree panel (navigation sidebar where tree `j`/`k` moves selection only and Enter on a file jumps diff to that file, switching the active file in single-file mode) with flat and hierarchical views, file type icons, lsd-style tree connector lines, directory collapsing, folder expand/collapse (`e` toggle, `1` tree focus toggle, `h`/`l` or `Ctrl-H`/`Ctrl-L` directional focus, `g`/`G`/`d`/`u` tree hunk navigation, scrollable, dynamic width adapting to content and terminal size, auto-hiding when terminal is too narrow, smart path truncation with `..` indicator and surrounded scroll that shifts indent origin to show deeply-nested cursor labels in full, default-visible for nested/larger diffs at 96+ terminal columns and default-hidden for small flat diffs or narrow terminals, file headers always visible), single-file view (`a` toggle independent of tree), an always-visible cursor line (tinted background bar, scrolloff=8), hidden-by-default status bar (appears for search, messages, visual mode, and help), visual line selection (`v`) for copying `path:line` references, and quick path copy (`c`/`C` for relative/absolute path). Hides whitespace-only changes by default (`--show-whitespace`/`-w` to include). Supports working tree (including untracked files by default, `--no-untracked` to hide), staged, commit, range diffs, and base-branch auto-detection (`--base`/`-b`, walks first-parent history to find the closest ancestor branch, matching `gbb`; works with remote-only branches). Invalid refs/ranges exit 1 with git stderr. Includes criterion microbenchmarks (`cargo bench`) for the render pipeline, diff parsing, word-level diffs, and tree building. Built by `apply.sh` and symlinked to `~/.cargo/bin/gd`. |
| `tui/` | Shared terminal UI library (Rust) providing ANSI utilities (strip, measure, split, wrap), syntax highlighting (syntect, GitHub Dark theme), and pager helpers (key mapping, terminal control, clipboard, editor delegation). Used by `md` and `gd` as a workspace dependency. |

### Claude Code Skills

| Skill | Description |
|-------|-------------|
| `slides/` | Slidev presentation management via the `/slides` slash command — create, author, dev, build, and export presentations in `~/Code/personal/slides/` |

### Shared Agent Configuration (Claude Authority)

`apply.sh` links Codex and Agents to Claude-authored sources so they stay in sync:

- `~/.codex/config.toml` → `configs/codex/config.toml`
- `~/.codex/rules/default.rules` → `configs/codex/rules/default.rules`
- `~/.codex/claude-rules` → `configs/claude/rules`
- `~/.codex/skills/<skill>` → `configs/claude/skills/<skill>`
- `~/.agents/skills/<skill>` → `configs/claude/skills/<skill>`

`configs/claude/CLAUDE.md` is symlinked to `~/.codex/AGENTS.md` (listed in the `configs/` table above). This keeps shared Codex guidance in the same Claude-authored source of truth.

Codex is configured with `approval_policy = "on-request"` in `configs/codex/config.toml`, which keeps routine work unblocked while still asking before higher-risk operations.

Codex runtime/session files remain machine-local and are not source-controlled: `~/.codex/auth.json`, `~/.codex/history.jsonl`, `~/.codex/log/`, `~/.codex/models_cache.json`, `~/.codex/sessions/`, `~/.codex/shell_snapshots/`, `~/.codex/tmp/`, and `~/.codex/version.json`.

## Useful Commands

| Command | Description |
|---------|-------------|
| `./apply.sh` | Run setup |
| `LOG_LEVEL=debug ./apply.sh` | Verbose setup |
| `reload` | Restart fish shell |
| `vd` | Open Neovim diff viewer (vs base branch) |
| `vd --staged` | Diff viewer for staged changes |
| `vd <ref>` | Diff viewer against a specific ref or commit |
| `/slides` | Manage Slidev presentations (create, dev, build, deploy, export) |
