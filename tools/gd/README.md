# gd

Terminal git diff viewer with syntax highlighting, word-level diff highlights, file type icons, and a built-in pager.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/gd`.

## Usage

```
gd                  # working tree changes (unstaged + untracked)
gd --staged         # staged changes
gd HEAD~1           # last commit's changes
gd main..HEAD       # range diff
gd --no-untracked   # hide untracked files
gd --no-pager       # print to stdout
gd --no-color       # disable ANSI colors
```

No changes exits cleanly (like `git diff`). Pager auto-activates when output exceeds terminal height. In working tree mode (bare `gd`), untracked files are shown as all-added diffs with `?` icon and `(Untracked)` header. Binary files (containing null bytes) and large files (>256KB) are skipped. Use `--no-untracked` to hide them.

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Scroll down one line |
| `k` / `Up` | Scroll up one line |
| `d` / `Ctrl-D` / `PageDown` | Half page down |
| `u` / `Ctrl-U` / `PageUp` | Half page up |
| `g` / `Home` | Top |
| `G` / `End` | Bottom |
| `Space` | Toggle cursor (underline indicator with scrolloff=8) |

### Diff Navigation

| Key | Action |
|-----|--------|
| `]c` / `[c` | Next/prev hunk |
| `]f` / `[f` | Next/prev file (switches single-file view when tree is visible) |
| `o` | Toggle full file context |

### Search

| Key | Action |
|-----|--------|
| `/` | Search |
| `n` / `N` | Next/prev match |

### File Tree

When the file tree is visible, the diff pane shows only the currently selected file. Use `]f`/`[f` or arrow keys in the tree to switch files. Closing the tree returns to the full concatenated diff view.

| Key | Action |
|-----|--------|
| `e` | Toggle file tree panel (open/focus/close cycle) |
| `Tab` | Switch focus to tree |
| `Ctrl-L` | Show + focus tree |
| `Ctrl-H` | Return focus to diff |
| `j` / `k` | Navigate and preview files (when tree focused) |
| `Enter` | Select file and show its diff (stays in tree) |
| `Esc` | Return focus to diff |

### Visual Mode

| Key | Action |
|-----|--------|
| `v` | Enter visual line selection mode |
| `j` / `k` | Extend selection down/up (clamped to current file) |
| `y` | Copy `path:start-end` to clipboard (via `pbcopy`) |
| `Esc` | Cancel selection |

### Other

| Key | Action |
|-----|--------|
| `E` | Open file in `$EDITOR` at current line |
| `?` | Help overlay |
| `q` / `Ctrl-C` | Quit |

## Architecture

**Render pipeline**: Runs `git diff`, parses into typed structs (`DiffFile`/`DiffHunk`/`DiffLine`), appends untracked files as synthetic all-added diffs in working tree mode, then renders all files as a single ANSI-colored document with dual line numbers, syntax highlighting (syntect, GitHub Dark theme), diff background colors, and word-level highlights (via `similar::TextDiff::from_words()`).

**Display format**: Dual line-number gutter (`old | new |`), `+`/`-` markers with colored backgrounds (green for added, red for deleted), brighter backgrounds on changed words within paired add/delete blocks, `↪` continuation markers on wrapped lines, and file/hunk header separators.

**Pager**: Alternate screen, raw mode, crossterm event loop. Supports scrolling, search with reverse-video highlighting, `]c`/`[c` hunk navigation, `]f`/`[f` file navigation, `$EDITOR` delegation with line-number positioning, a toggleable right-side file tree panel with flat and hierarchical views, file type icons, lsd-style tree connector lines (rounded corners, branch/end markers), and directory collapsing (hidden by default, `e` to toggle, scrollable, `Ctrl-H`/`Ctrl-L` directional focus), single-file view when tree is visible (diff pane shows only the selected file, `]f`/`[f` switches files), with auto-sync cursor tracking, centered viewport on hunk/file jumps, full-file context toggle (`o`), an optional cursor line (`Space` to toggle, underline indicator, scrolloff=8), and visual line selection mode (`v`) for copying `path:line` references to the clipboard.

## Modules

- `main.rs` — CLI parsing (clap), `DiffSource` resolution, git diff, render, pager decision
- `git/mod.rs` — Synchronous git command runner (`std::process::Command`)
- `git/diff.rs` — Unified diff parser with multi-hunk support
- `render.rs` — `DiffFile[]` → ANSI text with line numbers, syntax highlighting (via `tui::highlight`) + diff colors, word-level highlights
- `style.rs` — Diff color palette (GitHub Dark-inspired) and ANSI helpers
- `pager.rs` — Built-in pager with diff navigation, search, editor delegation
- `ansi.rs` — Re-exports from `tui::ansi`: `strip_ansi`, `visible_width`, `split_ansi`, `wrap_line_for_display`, `AnsiState`

## Build

```bash
cargo build --release   # from tools/gd/
```
