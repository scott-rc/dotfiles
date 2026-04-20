# Dotfiles

macOS dotfiles managed via symlinks.

## Quick Start

```bash
./apply.sh
```

This installs Homebrew, packages from `Brewfile`, Nix, and creates symlinks from config locations to this repo.

## Architecture

### lib.sh

Shared shell library sourced by `apply.sh` and `test_apply.sh`. Contains color constants, numeric log-level functions (`log_debug`, `log_info`, `log_warn`, `log_error`), section/success formatters (`log_section`, `log_success`), `run_with_spinner()` for background commands with animated progress indicators (bypassed in debug mode), and `ensure_symlink()`.

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
9. Builds CLI tools via Cargo workspace (md, tui, boom) and standalone gd from `~/Code/personal/gd/`
10. Uses sudo for `/etc/*` paths (e.g., nix.conf)

### Brewfile

All Homebrew packages are declared here and installed eagerly during setup.

### configs/

Configuration directories symlinked to their expected locations:

| Directory | Target |
|-----------|--------|
| `atuin/` | `~/.config/atuin/config.toml` |
| `bat/` | `~/.config/bat` |
| `claude/` | `~/.claude/{CLAUDE.md,settings.json,keybindings.json,skills,hooks,statusline,rules}` (individual symlinks per file/directory) |
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
| `nvim/` | `~/.config/nvim` — modular Neovim 0.12 config with vim.pack plugin manager, native LSP, native statusline, snacks.nvim (picker, explorer, scroll, indent), satellite.nvim, format-on-save (conform.nvim), codelens, undotree |
| `terminal/` | macOS Terminal.app color theme (GitHub Dark; imported manually, not symlinked) |
| `zed/` | `~/.config/zed/{settings.json,keymap.json}` |
| `zellij/` | `~/.config/zellij/{config.kdl,layouts}` (status bar via [zjstatus](https://github.com/dj95/zjstatus) WASM plugin) |
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
| `Cmd+Shift+E` | Toggle embed/float pane |
| `Cmd+Shift+F` | Toggle fullscreen |
| `Cmd+Shift+\\` | Toggle floating panes |
| `Cmd+Shift+,/.` | Previous/next swap layout (floating: staggered/enlarged/fullscreen/spread) |
| `Cmd+Shift+[/]` | Previous/next Zellij tab |
| `Cmd+Shift+I/O` | Move tab left/right |
| `Cmd+Shift++/-/=` | Resize increase/decrease |
| `Cmd+Shift+Up/Down` | Focus up/down |
| `Cmd+Shift+G` | Lock mode |
| `Cmd+Shift+P` | Pane mode |
| `Cmd+Shift+T` | New Zellij tab |
| `Cmd+Shift+Y` | Tab mode |
| `Cmd+Shift+S` | Session mode |
| `Cmd+Shift+U` | Enter scroll mode + half-page up |
| `Cmd+Shift+R` | Resize mode |
| `Cmd+Shift+M` | Move mode |

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

#### Neovim

| Shortcut | Action |
|----------|--------|
| `Ctrl+W r` | Enter resize mode (Hydra) — then `h/l` shrink/grow width, `j/k` grow/shrink height, `=` equalize, `Esc` exit |

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
| `gd` | Terminal git diff viewer — moved to its own repo at `~/Code/personal/gd/`. Built by `apply.sh` and symlinked to `~/.cargo/bin/gd`. |
| `boom/` | Kubernetes deploy tool (Rust) with template rendering (minijinja), manifest parsing, tiered resource application via kube-rs server-side apply, rollout monitoring with diagnostics, global-deploy for cluster-scoped resources, rolling restart via annotation patching, stale resource pruning, and colored terminal output. Supports `deploy`, `global-deploy`, `restart`, and `render` subcommands with configurable timeouts, exit codes (0 success, 1 failure, 70 timeout), and a `--prune` flag for cleanup. Built by `apply.sh` and symlinked to `~/.cargo/bin/boom`. |
| `tui/` | Shared terminal UI library (Rust) providing ANSI utilities (strip, measure, split, wrap), syntax highlighting (syntect, GitHub Dark theme), and pager helpers (key mapping, terminal control, clipboard, editor delegation). Used by `md` as a workspace dependency. |
| `claude-transcripts` | Python 3 (stdlib-only) parser for Claude Code session transcripts (`~/.claude/projects/`). Subcommands: `list` (sessions with duration, tokens, tools), `show <id>` (colored transcript with `--full`/`--thinking`/`--all`), `search <pattern>` (cross-session text search), `stats` (aggregate token usage, model/tool/project breakdowns, daily activity). |

### Claude Code Skills

| Skill | Description |
|-------|-------------|
| `brainstorm/` | Interviews the user relentlessly about a plan or design, walking each branch of the decision tree until shared understanding is reached. |
| `code/` | Writes, reviews, tests, and optimizes code, and designs architectural refactors — enforces TDD for new features and bug fixes, runs code review, benchmarks, and mutation testing. |
| `compose/` | Creates, updates, reviews, and improves Claude Code skills, CLAUDE.md rules, and session prompts, and writes handoff files for session continuity. |
| `git/` | Handles git commits, pushes, PRs, rebases, CI triage and monitoring, code review, branch splitting with stacked PRs via git-spice, and GitHub interactions. |
| `plan/` | Turns a Brief-populated plan file into phased work (`create`), executes it phase-by-phase with commit checkpoints (`execute`), and retrospects with auto-proposed Fixup phases on completion (`review`). |
| `prd/` | Creates a PRD through user interview, codebase exploration, and module design, saved as the Brief section of a plan file at `./tmp/<name>/plan.md`. |
| `slack-messaging/` | Enforces Slack formatting rules and tool selection when sending Slack messages via the Slack MCP integration. |
| `ubiquitous-language/` | Extracts a DDD-style ubiquitous language glossary from the current conversation, flagging ambiguities and proposing canonical terms; saves to `UBIQUITOUS_LANGUAGE.md`. |

Each skill follows this directory structure:

```
<skill-name>/
├── SKILL.md           # Hub/router — routes input to operations
├── operations/        # Operation files (one per workflow)
├── references/        # Shared knowledge and guidelines (optional)
└── scripts/           # Shell scripts (optional)
```

### Shared Configuration (Claude Authority)

`apply.sh` links Codex to Claude-authored sources so they stay in sync:

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
| `bash test_apply.sh` | Run lib.sh unit tests |
| `reload` | Restart fish shell |
