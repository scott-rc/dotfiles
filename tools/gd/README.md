# gd

Terminal git diff viewer with embedded neovim for syntax highlighting and full vim navigation.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/gd`.

## Usage

```
gd                  # working tree changes (unstaged)
gd --staged         # staged changes
gd HEAD~1           # last commit's changes
gd main..HEAD       # range diff
```

No changes exits cleanly (like `git diff`).

## Keybindings

| Key | Action |
|-----|--------|
| `]c` / `[c` | Next/prev hunk |
| `]f` / `[f` | Next/prev changed file |
| `q` | Quit |
| `Ctrl-C` | Force quit |

All standard vim navigation works (hjkl, /, gg, G, etc.).

## Architecture

**Event loop**: `tokio::select!` merging crossterm terminal events, nvim redraw notifications, and app commands (file navigation, quit) via RPC.

**Nvim bridge**: Spawns `nvim --embed -u NONE`, attaches UI with `ext_linegrid`, receives redraw events via `NvimHandler` trait, maintains a `GridBuffer` (2D cell array + highlight attrs), renders into ratatui via `NvimWidget`. App commands (`gd_cmd`) are routed from nvim keymaps via `vim.rpcnotify`.

**Diff pipeline**: Runs `git diff --unified=999999` per file, parses into typed structs (`DiffFile`/`DiffHunk`/`DiffLine`), loads content into nvim buffer (working tree files via `:edit`, other sources via `git show` into scratch buffers), applies extmarks for line-level highlights (`GdAdded`/`GdDeleted`) and virtual lines for deleted content. Word-level diffs via `similar::TextDiff::from_words()` produce `GdAddedWord`/`GdDeletedWord` highlights.

**Dual line numbers**: Custom `statuscolumn` function reads `b:gd_line_map` (real lines) and `b:gd_virt_map` (virtual lines) to render old/new line numbers with `GdGutterNum`/`GdGutterSep` highlight groups.

**Staging** (planned): Builds minimal unified diff patches from selected lines, pipes through `git apply --cached` (or `--reverse` for unstage).

## Modules

- `main.rs` — CLI parsing (clap), tokio bootstrap, orchestration
- `app.rs` — `DiffSource` enum, `FileList` navigation, CLI arg resolution
- `event.rs` — `AppEvent`/`AppCmd` enums, `tokio::select!` loop
- `theme.rs` — Diff highlight group definitions (`GdAdded`, `GdDeleted`, etc.)
- `git/mod.rs` — Async git command runner
- `git/diff.rs` — Unified diff parser (`DiffFile`/`DiffHunk`/`DiffLine`)
- `nvim/mod.rs` — Spawn nvim, `ui_attach`, lifecycle, diff rendering pipeline (`load_diff`, `setup_statuscolumn`)
- `nvim/bridge.rs` — Handler trait, redraw event parsing, `gd_cmd` RPC routing
- `nvim/grid.rs` — `GridBuffer` (cells + highlight attrs)
- `nvim/input.rs` — `KeyEvent` → nvim key notation
- `ui/layout.rs` — Terminal region splitting
- `ui/nvim_widget.rs` — `GridBuffer` → ratatui Widget

### Planned

- `git/stage.rs` — Patch construction + git apply
- `git/watch.rs` — File watcher → event channel
- `ui/file_tree.rs` — File tree sidebar
- `ui/scroll_gutter.rs` — Change-position marks
- `ui/status_bar.rs` — Bottom status bar
- `ui/picker.rs` — Diff source picker overlay

## Build

```bash
cargo build --release   # from tools/gd/
```
