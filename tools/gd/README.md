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

No changes exits cleanly (like `git diff`). Invalid refs/ranges now exit with status 1 and print the underlying `git diff` error to stderr. Pager auto-activates when output exceeds terminal height. In working tree mode (bare `gd`), untracked files are shown as all-added diffs with `?` icon and `(Untracked)` header. Binary files (containing null bytes) and large files (>256KB) are skipped. Use `--no-untracked` to hide them. Files are sorted by path so all-files view and tree order always match.

## Keybindings

All keys work the same regardless of what's visible. No modes, no context-dependent behavior (except search input, which naturally captures typed characters).

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Scroll down (skips headers) |
| `k` / `Up` | Scroll up (skips headers) |
| `d` | Half page down |
| `u` | Half page up |
| `g` / `Home` | Top |
| `G` / `End` | Bottom |
| `z` | Center viewport on cursor |

### Diff Navigation

| Key | Action |
|-----|--------|
| `]` / `[` | Next / previous hunk |
| `}` / `{` | Next / previous file (switches file in single-file mode) |
| `s` | Toggle single-file view |
| `o` | Toggle full file context |

### Search

| Key | Action |
|-----|--------|
| `/` | Enter search |
| `n` / `N` | Next / previous match |
| `Enter` | Submit search (in search input) |
| `Esc` / `Ctrl-C` | Cancel search (in search input) |

### Selection

| Key | Action |
|-----|--------|
| `v` | Start visual selection at cursor line |
| `y` | Yank `path:start-end` from selection to clipboard (via `pbcopy`) |

### Other

| Key | Action |
|-----|--------|
| `l` | Toggle file tree sidebar |
| `e` | Open file in `$EDITOR` at current line |
| `?` | Toggle keybinding hints bar |
| `q` / `Ctrl-C` | Quit |

## Architecture

**Render pipeline**: Runs `git diff`, parses into typed structs (`DiffFile`/`DiffHunk`/`DiffLine`), appends untracked files as synthetic all-added diffs in working tree mode, then renders all files as a single ANSI-colored document with dual line numbers, syntax highlighting (syntect, GitHub Dark theme), diff background colors, and word-level highlights (via `similar::TextDiff::from_words()`).

**Display format**: Dual line-number gutter (`old | new |`), `+`/`-` markers with colored backgrounds (green for added, red for deleted), brighter backgrounds on changed words within paired add/delete blocks, continuation markers on wrapped lines, and file/hunk header separators.

**Pager**: Alternate screen, raw mode, crossterm event loop. Lives under `src/pager/` with focused submodules. Uses a flat keymap with no context-dependent keys -- every key always does the same thing. The tree panel is passive (auto-syncs to cursor position, no focus mode), defaults visible for nested or larger diffs (requires 96+ terminal columns), and defaults hidden for small flat diffs or narrow terminals. Tree width adapts dynamically to content and terminal size, auto-hiding when the terminal is too narrow to fit both an 80-column diff area and a 15-column tree panel. The user can still manually toggle the tree with `l` on narrower terminals. Long file paths are truncated with a `..` indicator. Paths that fit within the panel are never truncated. When the cursor is on a deeply nested file whose label overflows, the tree shifts its indent origin rightward to show the full label, displaying a `..` indicator on shallower entries whose connectors are scrolled off. File headers remain visible even when the tree is open. Selection uses visual select (`v` to anchor, `y` to yank). A toggleable tooltip bar (`?`) shows available keybindings at the bottom of the screen. The status bar shows position indicators, single-file info, and mark status.

## Modules

- `main.rs` -- CLI parsing (clap), `DiffSource` resolution (including `--base`/`-b` base-branch detection), git diff, render, pager decision
- `git/mod.rs` -- Synchronous git command runner (`std::process::Command`)
- `git/diff.rs` -- Unified diff parser with multi-hunk support
- `render.rs` -- `DiffFile[]` -> ANSI text with line numbers, syntax highlighting (via `tui::highlight`) + diff colors, word-level highlights
- `style.rs` -- Diff color palette (GitHub Dark-inspired) and ANSI helpers
- `pager/mod.rs` -- Pager entrypoint and shared wiring
- `pager/content.rs` -- Pure line-map helpers (next_content_line, snap_to_content, etc.)
- `pager/text.rs` -- Char-boundary helpers for search input
- `pager/types.rs` -- Typed pager enums/newtypes and action identifiers
- `pager/keymap.rs` -- Keybinding table + help-line generation
- `pager/state.rs` -- Pager/document state model and remap helpers
- `pager/navigation.rs` -- Navigation (hunk/file jumping, viewport, sync_tree_cursor)
- `pager/search.rs` -- Search overlay (submit, cancel, handle_search_key, scroll_to_match)
- `pager/tree.rs` -- File tree (build_tree_entries, build_tree_lines, TreeEntry)
- `pager/rendering.rs` -- Rendering (render_screen, tooltip bar, format_status_bar, scrollbar)
- `pager/reducer.rs` -- Flat reducer (handle_key, dispatch_normal_action)
- `pager/runtime.rs` -- Run loop (run_pager, re_render, regenerate_files)
- `ansi.rs` -- Re-exports from `tui::ansi`: `visible_width`, `split_ansi`, `wrap_line_for_display` (plus `strip_ansi` for tests)

## Build

```bash
cargo build --release   # from tools/gd/
```

## Coverage

Requires [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) (`cargo install cargo-llvm-cov`).

```bash
./coverage.sh           # HTML report, opens in browser
./coverage.sh lcov      # write lcov.info (for Coverage Gutters / IDE integration)
./coverage.sh text      # terminal summary table
./coverage.sh json      # LLVM JSON export to coverage.json
./coverage.sh agent     # lcov.info + machine-parseable summary (see below)
```

### Agent workflow

`./coverage.sh agent` produces three sections on stdout, designed for AI agents to parse:

- **COVERAGE SUMMARY** -- per-file line/function coverage percentages
- **UNCOVERED FUNCTIONS** -- `file:line function_name` for every function with 0 hits (test files excluded)
- **UNCOVERED LINES BY FILE** -- `file (uncovered: 10-15,22,30-35)` with collapsed ranges (test files excluded)

It also writes `lcov.info`, which agents can read directly for line-granular data. The lcov format uses `SF:` for source file, `DA:line,count` for line hits (0 = uncovered), and `FNDA:count,name` for function hits.

## Debug tracing

Set `GD_DEBUG=1` to emit structured debug output to stderr for rerender and regenerate paths (e.g. `GD_DEBUG=1 gd`). Default: no debug I/O. Useful for diagnosing view state after document swaps.
