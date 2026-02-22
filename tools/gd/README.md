# gd

Terminal git diff viewer with syntax highlighting, word-level diff highlights, file type icons, and a built-in pager.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/gd`.

## Usage

```
gd                  # working tree changes (unstaged + untracked)
gd --staged         # staged changes
gd --base / -b      # diff against auto-detected base branch
gd HEAD~1           # last commit's changes
gd main..HEAD       # range diff
gd --no-untracked   # hide untracked files
gd --no-pager       # print to stdout
gd --no-color       # disable ANSI colors
```

No changes exits cleanly (like `git diff`). Invalid refs/ranges now exit with status 1 and print the underlying `git diff` error to stderr. Pager auto-activates when output exceeds terminal height. In working tree mode (bare `gd`), untracked files are shown as all-added diffs with `?` icon and `(Untracked)` header. Binary files (containing null bytes) and large files (>256KB) are skipped. Use `--no-untracked` to hide them.

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Next content line (skips headers) |
| `k` / `Up` | Previous content line (skips headers) |
| `Ctrl-D` / `PageDown` | Half page down (snaps to content) |
| `Ctrl-U` / `PageUp` | Half page up (snaps to content) |
| `g` / `Home` | First content line |
| `G` / `End` | Last content line |

### Diff Navigation

| Key | Action |
|-----|--------|
| `d` / `u` | Next/prev hunk (change group in full-context mode) |
| `D` / `U` | Next/prev file (shows file position) |
| `a` | Toggle single-file view (independent of tree panel) |
| `Space` | Toggle full file context |

### Search

| Key | Action |
|-----|--------|
| `/` | Search |
| `n` / `N` | Next/prev match |

### File Tree

The file tree is a navigation aid that does not affect the diff pane's content. When focused, selecting a file scrolls the diff to that file. Folders are focusable and can be expanded/collapsed with Enter. File headers are hidden in the diff pane while the tree is visible, since the tree already shows file names. Single-file view can be toggled independently with `a`.

| Key | Action |
|-----|--------|
| `e` | Toggle file tree panel (open/focus/close cycle) |
| `Tab` | Switch focus to tree |
| `1` | Toggle tree focus |
| `l` / `Ctrl-L` | Show + focus tree |
| `h` / `Ctrl-H` | Return focus to diff |
| `j` / `k` | Navigate tree and scroll diff to file (when tree focused) |
| `g` / `G` | Jump to first/last file (when tree focused) |
| `d` / `u` | Next/prev hunk (global, syncs tree selection) |
| `Enter` | Scroll to file / toggle folder expand/collapse |
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

**Pager**: Alternate screen, raw mode, crossterm event loop. Opens with the file tree visible and focused, all files shown with hunk context (surrounding diff lines only). Supports scrolling with content-line skipping (cursor skips file headers, hunk headers, and blank separators, landing only on Added/Deleted/Context lines), search highlighting with UTF-8-safe cursor editing, `d`/`u` hunk navigation (change-group navigation in full-context mode), `D`/`U` file navigation (with file position status), `$EDITOR` delegation with line-number positioning (diff paths resolved against repo root, so `E` works from subdirectories), a toggleable right-side file tree panel (navigation sidebar that scrolls diff to selected file without filtering) with flat and hierarchical views, file type icons, lsd-style tree connector lines (rounded corners, branch/end markers), directory collapsing (`e` to cycle open/focus/close, scrollable, `h`/`l` or `Ctrl-H`/`Ctrl-L` directional focus, `g`/`G`/`d`/`u` tree navigation), focusable folder nodes with Enter to expand/collapse, `1` for tree focus toggle, accent-colored separator when tree is focused, tree-focused key passthrough (search, hunk/file nav, help, editor, visual mode fall through to normal handlers), single-file view (`a` toggle independent of tree; diff pane shows only the selected file), with auto-sync cursor tracking, always-visible cursor line (tinted background bar, scrolloff=8), centered viewport on hunk/file jumps, full-file context toggle (`Space`, shows a scrollbar with colored change markers and viewport thumb), hidden-by-default status bar (appears for search input, transient messages, visual mode, and help overlay), and visual line selection mode (`v`) for copying `path:line` references to the clipboard.

## Modules

- `main.rs` — CLI parsing (clap), `DiffSource` resolution (including `--base`/`-b` base-branch detection), git diff, render, pager decision
- `git/mod.rs` — Synchronous git command runner (`std::process::Command`)
- `git/diff.rs` — Unified diff parser with multi-hunk support
- `render.rs` — `DiffFile[]` → ANSI text with line numbers, syntax highlighting (via `tui::highlight`) + diff colors, word-level highlights
- `style.rs` — Diff color palette (GitHub Dark-inspired) and ANSI helpers
- `pager.rs` — Built-in pager with diff navigation, search, editor delegation
- `ansi.rs` — Re-exports from `tui::ansi`: `visible_width`, `split_ansi`, `wrap_line_for_display` (plus `strip_ansi` for tests)

## Build

```bash
cargo build --release   # from tools/gd/
```
