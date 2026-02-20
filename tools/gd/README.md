# gd

Terminal git diff viewer with syntax highlighting, word-level diff highlights, and a built-in pager.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/gd`.

## Usage

```
gd                  # working tree changes (unstaged)
gd --staged         # staged changes
gd HEAD~1           # last commit's changes
gd main..HEAD       # range diff
gd --no-pager       # print to stdout
gd --no-color       # disable ANSI colors
```

No changes exits cleanly (like `git diff`). Pager auto-activates when output exceeds terminal height.

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Scroll down one line |
| `k` / `Up` | Scroll up one line |
| `d` / `Space` / `Ctrl-D` / `PageDown` | Half page down |
| `u` / `Ctrl-U` / `PageUp` | Half page up |
| `g` / `Home` | Top |
| `G` / `End` | Bottom |

### Diff Navigation

| Key | Action |
|-----|--------|
| `]c` / `[c` | Next/prev hunk |
| `]f` / `[f` | Next/prev file |

### Search

| Key | Action |
|-----|--------|
| `/` | Search |
| `n` / `N` | Next/prev match |

### File Tree

| Key | Action |
|-----|--------|
| `T` | Toggle file tree panel |
| `Tab` | Switch focus between diff and tree |
| `j` / `k` | Navigate files (when tree focused) |
| `Enter` | Jump to file (when tree focused) |
| `Esc` | Return focus to diff (when tree focused) |

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
| `e` | Open file in `$EDITOR` at current line |
| `?` | Help overlay |
| `q` / `Ctrl-C` | Quit |

## Architecture

**Render pipeline**: Runs `git diff`, parses into typed structs (`DiffFile`/`DiffHunk`/`DiffLine`), renders all files as a single ANSI-colored document with dual line numbers, syntax highlighting (syntect, GitHub Dark theme), diff background colors, and word-level highlights (via `similar::TextDiff::from_words()`).

**Display format**: Dual line-number gutter (`old | new |`), `+`/`-` markers with colored backgrounds (green for added, red for deleted), brighter backgrounds on changed words within paired add/delete blocks, and file/hunk header separators.

**Pager**: Alternate screen, raw mode, crossterm event loop. Supports scrolling, search with reverse-video highlighting, `]c`/`[c` hunk navigation, `]f`/`[f` file navigation, `$EDITOR` delegation with line-number positioning, a toggleable right-side file tree panel with auto-sync cursor tracking, and visual line selection mode (`v`) for copying `path:line` references to the clipboard.

## Modules

- `main.rs` — CLI parsing (clap), `DiffSource` resolution, git diff, render, pager decision
- `git/mod.rs` — Synchronous git command runner (`std::process::Command`)
- `git/diff.rs` — Unified diff parser with multi-hunk support
- `render.rs` — `DiffFile[]` → ANSI text with line numbers, syntax + diff colors, word-level highlights
- `style.rs` — Diff color palette (GitHub Dark-inspired) and ANSI helpers
- `pager.rs` — Built-in pager with diff navigation, search, editor delegation
- `ansi.rs` — `strip_ansi`, `visible_width`, `split_ansi`, `wrap_line_for_display`

## Build

```bash
cargo build --release   # from tools/gd/
```
