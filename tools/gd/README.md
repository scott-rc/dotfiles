# gd

Terminal git diff viewer with syntax highlighting, word-level diff highlights, file type icons, and a built-in pager. Optional browser-based viewer via `--web`.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/gd`.

## Usage

```
gd                  # working tree changes (unstaged + untracked)
gd --staged         # staged changes
gd --base / -b      # diff against auto-detected base branch
gd HEAD~1           # last commit's changes
gd main..HEAD       # range diff
gd --no-untracked   # hide untracked files
gd --show-whitespace / -w  # include whitespace-only changes (hidden by default)
gd --no-pager       # print to stdout
gd --no-color       # disable ANSI colors
gd --theme light    # light theme for syntax highlighting (TUI mode)
gd --theme dark     # dark theme for syntax highlighting (TUI mode)
gd --web             # open interactive diff viewer in browser (requires --features web)
gd --web --no-open   # start server without opening browser (prints URL to stderr)
gd --web --port 3845 # use fixed port (default is random to prevent stale tab reconnection)
gd --web -b          # browser viewer for base branch diff
gd --replay ']]]]q' # replay keystrokes without a TTY (for benchmarking)
gd --replay ']]q' --cols 80 --rows 24  # replay with custom terminal size
```

The pager auto-reloads after returning from `$EDITOR` and when `.git/index` changes externally (e.g. staging in another terminal). Press `R` to manually reload. No changes exits cleanly (like `git diff`). Invalid refs/ranges now exit with status 1 and print the underlying `git diff` error to stderr. Pager auto-activates when output exceeds terminal height. Whitespace-only changes are hidden by default (`-w` passed to `git diff`); use `--show-whitespace` to include them. `--base`/`-b` works even if the base branch only exists as a remote tracking ref (falls back to `origin/<branch>`). In working tree mode (bare `gd`), untracked files are shown as all-added diffs with `?` icon and `(Untracked)` header. Binary files (containing null bytes) and large files (>256KB) are skipped. Use `--no-untracked` to hide them. Files are sorted by path so all-files view and tree order always match.

`--web` starts a local HTTP server and opens an interactive diff viewer in the browser. The same diff parsing, syntax highlighting, and word-level highlight logic is reused — HTML spans replace ANSI codes, and browser CSS/JS replaces terminal Phase 2 layout. The browser view supports keyboard navigation (same keys as the terminal pager), a file tree sidebar (on the right, resizable by dragging the border), search, single-file/all-files view, and auto-reloads when `.git/index` changes. The file tree renders box-drawing connectors (`├──`, `└──`, `│`) matching the TUI, per-file change stats (`+N -N` additions/deletions next to each file, aggregated on directories). Hovering a file in the tree highlights the corresponding file header in the diff pane. Long file labels truncate with ellipsis; hovering shows the full name in a tooltip. Theme support: press `T` to cycle between system/light/dark modes (persisted to localStorage). File icons use Nerd Font codepoints (same as TUI) with a CSS font stack that picks the first available Nerd Font; if none installed, icons show as placeholder characters. Requires `--features web` at build time; without the feature the flag prints an error and exits. The server binds to a random port by default (prevents stale browser tabs from reconnecting after restart); use `--port 3845` for a fixed port. Data flows over a WebSocket. The server exits automatically 2 seconds after the last browser tab closes (handles page refresh without immediate exit).

## Keybindings

All keys work the same regardless of what's visible. No modes, no context-dependent behavior (except search input, which naturally captures typed characters).

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Scroll down (skips headers) |
| `k` / `Up` | Scroll up (skips headers) |
| `d` | Half page down |
| `u` | Half page up |
| `g` / `Home` | Top (diff or tree, based on focus) |
| `G` / `End` | Bottom (diff or tree, based on focus) |
| `z` | Center viewport on cursor |

### Diff Navigation

| Key | Action |
|-----|--------|
| `]` / `[` | Next / previous hunk |
| `}` / `{` | Next / previous visible file (skips collapsed dirs when tree visible) |
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
| `c` | Copy relative file path to clipboard |
| `C` | Copy absolute file path to clipboard |

### Tree

| Key | Action |
|-----|--------|
| `l` | Toggle file tree sidebar |
| `t` | Toggle tree / diff focus |
| `1` / `Cmd-1` | Focus diff pane |
| `Cmd-E` | Open & focus tree, or close if already focused |
| `Enter` / `Space` | Show file / toggle directory (in tree focus, keeps tree focused) |
| `za` | Toggle collapse |
| `zA` | Toggle collapse recursive |

### Other

| Key | Action |
|-----|--------|
| `e` | Open file in `$EDITOR` at current line |
| `R` | Reload diff |
| `?` | Toggle keybinding hints bar |
| `T` | Cycle theme: system → light → dark (web only) |
| `q` / `Ctrl-C` | Quit |

### Staging

| Key | Action |
|-----|--------|
| `a` | Stage line or visual selection (unstage in `--staged` view) |
| `A` | Stage hunk (unstage hunk in `--staged` view) |
| `x` | Discard/revert line or visual selection |
| `X` | Discard/revert hunk |

## Architecture

**Render pipeline**: Two-phase architecture separating expensive styling from cheap layout. **Phase 1 (style)**: Runs `git diff`, parses into typed structs (`DiffFile`/`DiffHunk`/`DiffLine`), appends untracked files as synthetic all-added diffs in working tree mode. Each file is then styled in parallel using work-stealing (atomic index, largest-file-first) -- syntax highlighting (syntect, GitHub Dark theme), diff background colors, and word-level highlights (via `similar::TextDiff`) are applied eagerly to produce width-independent `StyledLine` data. **Phase 2 (layout)**: Takes styled content and a target width, then wraps lines, assembles gutters, generates file headers and hunk separators, and builds the display-line array and metadata (`LineInfo`, `file_starts`, `hunk_starts`). On startup, tree width is pre-computed so Phase 2 runs once at the correct width. On width changes (tree toggle, terminal resize), only Phase 2 re-runs -- Phase 1 output is preserved, making relayout nearly instant. On content changes (staging, reloading), both phases run. Rendering uses dual line numbers, `+`/`-` markers, and word-level highlights within paired add/delete blocks.

**Display format**: Dual line-number gutter (`old | new |`), `+`/`-` markers with colored backgrounds (green for added, red for deleted), brighter backgrounds on changed words within paired add/delete blocks, continuation markers on wrapped lines, file header separators, and dim dashed-line hunk separators between hunks within a file.

**Web mode** (`--web`, behind `features = ["web"]`): An alternative rendering path that reuses the same git diff parsing and word-level highlight logic but outputs HTML instead of ANSI. `highlight_line_html()` in `tui::highlight` produces `<span style="...">` tokens; `html_render.rs` applies word-level `<mark>` tags at the correct byte positions. File/directory icons use the same Nerd Font codepoints as the TUI (`style::file_icon`, `style::dir_icon`) with ANSI colors converted to CSS via `ansi_to_css()`. Data is serialized as JSON and pushed to the browser over a WebSocket. The frontend (`web_assets/`) uses **virtual rendering** for large diffs: pre-computes line offsets, only creates DOM elements for visible lines plus a buffer (~100 lines total), and re-renders on scroll. This reduces DOM node count from 160K+ to ~1.5K on 10K-line diffs, with sub-5ms navigation latency. The same keyboard navigation as the terminal pager is supported. Light/dark/system themes use CSS custom properties (`T` key cycles, persisted to localStorage). Tree panel is resizable (drag handle, width persisted to localStorage). A tokio task polls `.git/index` mtime every 2 seconds and broadcasts updates to all connected clients.

**Pager**: Alternate screen, raw mode, crossterm event loop. Lives under `src/pager/` with focused submodules. Supports interactive staging, unstaging, and reverting of individual diff lines and entire hunks. Startup heuristics pick sensible defaults: single-file diffs with 3 or fewer hunks open in full-context mode (showing the entire file); diffs with more than 5 files, or multi-file diffs whose rendered output exceeds 3x the terminal height, open in single-file view with full context enabled (viewing one file at a time benefits from seeing the full file). Manual toggles (`s` for view scope, `o` for full context) override the heuristics permanently for the session. Uses a flat keymap with no context-dependent keys -- every key always does the same thing. The tree panel supports two focus states (`t` toggles): when focused, the tree cursor is bold/highlighted and the diff pane dims; when unfocused, the tree cursor shows a subtle background. `Enter`/`Space` in tree focus shows a file or toggles a directory (keeping tree focused), `za`/`zA` collapse single or recursive, and `}`/`{` jump to the next/prev visible file in the tree. The tree defaults visible for nested or larger diffs (requires 96+ terminal columns), and defaults hidden for small flat diffs or narrow terminals. Tree width adapts dynamically to content and terminal size, auto-hiding when the terminal is too narrow to fit both an 80-column diff area and a 15-column tree panel, and auto-showing when the terminal is resized wide enough. Explicit user toggles (`l`) are respected — if the user hides the tree, it stays hidden regardless of terminal width. The user can still manually toggle the tree with `l` on narrower terminals. Long file paths are truncated with a `..` indicator. Paths that fit within the panel are never truncated. When the cursor is on a deeply nested file whose label overflows, the tree shifts its indent origin rightward to show the full label, displaying a `..` indicator on shallower entries whose connectors are scrolled off. File headers remain visible even when the tree is open. Selection uses visual select (`v` to anchor, `y` to yank). A toggleable tooltip bar (`?`) shows available keybindings at the bottom of the screen. The status bar shows a position indicator (TOP / END / %) on the right. In single-file mode it also shows a file-type icon, the dimmed file path, and a `< N/total >` chevron counter on the left. In single-file mode, the pager remembers the cursor and scroll position for each file, restoring them when switching back. Positions are cleared when the document is regenerated (e.g. after staging or reloading).

## Modules

- `main.rs` -- CLI parsing (clap), `DiffSource` resolution (including `--base`/`-b` base-branch detection), git diff, render, pager decision
- `debug.rs` -- `GD_DEBUG=1` phase timing helpers (zero-cost when disabled)
- `git/mod.rs` -- Synchronous git command runner (`std::process::Command`)
- `git/diff.rs` -- Unified diff parser with multi-hunk support
- `git/patch.rs` -- Patch generation for line-level staging (selected lines to unified diff format)
- `render/mod.rs` -- Two-phase render pipeline: Phase 1 (`style_files`) produces width-independent styled content in parallel; Phase 2 (`layout`) wraps and assembles at target width. Syntax highlighting (via `tui::highlight`), diff colors
- `render/word_diff.rs` -- Word-level diff utilities: tokenization, change block detection, per-line highlight ranges, ANSI color application
- `style.rs` -- Diff color palette (GitHub Dark-inspired) and ANSI helpers
- `pager/mod.rs` -- Pager entrypoint and shared wiring
- `pager/content.rs` -- Pure line-map helpers (next_content_line, snap_to_content, etc.)

- `pager/types.rs` -- Typed pager enums/newtypes and action identifiers
- `pager/keymap.rs` -- Keybinding table + help-line generation
- `pager/state.rs` -- Pager/document state model and remap helpers
- `pager/navigation.rs` -- Navigation (hunk/file jumping, viewport, sync_tree_cursor)
- `pager/search.rs` -- Search overlay (submit, cancel, handle_search_key, scroll_to_match)
- `pager/tree.rs` -- File tree (build_tree_entries, build_tree_lines, TreeEntry)
- `pager/rendering.rs` -- Rendering (render_screen, tooltip bar, format_status_bar, scrollbar)
- `pager/reducer.rs` -- Flat reducer (handle_key, dispatch_normal_action)
- `pager/runtime.rs` -- Run loop (run_pager, re_render/full_render, regenerate_files)
- `ansi.rs` -- Re-exports from `tui::ansi`: `visible_width`, `split_ansi`, `wrap_line_for_display` (plus `strip_ansi` for tests)
- `web/mod.rs` -- Web mode entrypoint (`run_web_server`), gated behind `features = ["web"]`
- `web/server.rs` -- axum HTTP + WebSocket server, `.git/index` file watcher, auto-reload broadcast
- `web/html_render.rs` -- DiffFile → WebDiffFile HTML rendering (syntax highlighting + word-level highlights as HTML spans)
- `web/protocol.rs` -- Serializable WebSocket message types (`ServerMessage`, `WebDiffFile`, etc.)
- `web_assets/` -- Embedded frontend (index.html, style.css, app.js) served via `include_str!`

## Build

```bash
cargo build --release                # from tools/gd/ — default (terminal only)
cargo build --release --features web # from tools/gd/ — includes --web browser viewer
```

## E2E Tests

Browser-based Playwright tests for `--web` mode live in `e2e/`. Requires the web feature and a test git repo fixture.

```bash
cd e2e
npm install                    # install Playwright
npx playwright install chromium # install browser
./fixtures/setup.sh            # create test repo with known diff state
npm test                       # run all tests
npm run test:headed            # run with visible browser
```

Tests cover navigation (j/k/g/G/d/u), file tree (toggle, focus, click), search (open/close, n/N), and keybindings (help overlay, view toggles). The server's quit-on-close behavior ensures clean test teardown.

### Screenshot Tests

Visual regression tests using Playwright's built-in `toHaveScreenshot()`. Baseline PNGs are stored in `tests/screenshots.spec.ts-snapshots/`. Screenshots cover:

- Initial load (tree visible, all-files view)
- Tree hidden/focused states
- Help overlay, search overlay, single-file view
- Visual selection (single line and extended)

To update baselines after intentional UI changes:

```bash
npm test -- --grep "Screenshots" --update-snapshots
```

### Performance Profiling

The web UI exposes `window.__gdPerf` for agent-driven performance analysis. Query `__gdPerf.getMetrics()` in the browser console or via Chrome DevTools MCP tools (`evaluate_script`) to get timing data for initial load, render cycles, and navigation latency. See CLAUDE.md for the full profiling workflow.

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

## Benchmarking

### End-to-end startup time

Use `--replay q` to benchmark full startup (git commands + render pipeline + pager init) with color enabled. **Do not use `--no-pager`** for startup benchmarks -- it disables color when stdout is piped, hiding the dominant cost (syntax highlighting).

```bash
# Startup benchmark on a large diff (preferred)
hyperfine 'gd -b --replay q'

# With phase-level timing breakdown
GD_DEBUG=1 gd -b --replay q 2>timing.txt >/dev/null && cat timing.txt
```

Typical phase breakdown on a ~65-file diff:

| Phase | Typical time |
|-------|-------------|
| Git commands (base detect + diff) | ~65ms |
| Full-context re-diff (`-U999999`) | ~25ms |
| `style_files` (Phase 1, syntax + word diffs) | ~230ms |
| `layout` (Phase 2, wrapping + gutters) | ~70ms |

`style_files` dominates -- it runs syntect per-line and is sensitive to the largest file in the diff. Uses work-stealing parallelism (atomic index + largest-file-first ordering) to minimize load imbalance across cores.

### Microbenchmarks

```bash
cargo bench
```

Criterion benchmarks in `benches/bench.rs` cover diff parsing, the render pipeline, word-level diffs, and tree building. Results are stored in `target/criterion/` with HTML reports.

Compare against a baseline after making changes:

```bash
cargo bench --bench bench -- --save-baseline before
# ... make changes ...
cargo bench --bench bench -- --baseline before
```

### Profiling

Generate a flamegraph with [samply](https://github.com/mstange/samply):

```bash
cargo build --release
samply record ./target/release/gd --replay ']]]]q' HEAD~1
```

This opens the Firefox Profiler with a call tree and flame chart. Look for hot functions in `render/mod.rs` (`style_files`), `render/word_diff.rs` (`word_highlights`, `tokenize`), and `pager/tree.rs` (`build_tree_entries`).

## Replay mode

`--replay <KEYS>` drives the full pager pipeline (handle_key, render_screen, prefetch) without a TTY. Renders to an in-memory buffer; no terminal setup required. Combined with `GD_DEBUG=1`, emits per-keystroke timing to stderr. Defaults to 120x50; override with `--cols` and `--rows`.

```bash
GD_DEBUG=1 gd --replay ']]]]q' HEAD~1 2>timing.jsonl
```

Key format: plain chars map to keys (`]`, `q`, `j`). Special keys use angle brackets: `<Enter>`, `<Esc>`, `<Up>`, `<Down>`, `<C-c>`, `<Home>`, `<End>`, `<PgUp>`, `<PgDn>`, `<Tab>`, `<BS>`. Backslash escapes: `\n` (Enter), `\\` (literal backslash), `\<` (literal `<`).

## Debug tracing

Set `GD_DEBUG=1` to emit structured debug output to stderr. Default: no debug I/O. Two trace formats:

- **Phase timing** (`[gd:timing]`): Emitted from `main`, `render`, and `pager` modules at each startup phase boundary. Shows cumulative ms from process start. Useful for identifying which phase dominates startup time.
- **Pager events** (`[gd]`): JSON traces for rerender, regenerate, per-keystroke timing, and view state. Useful for diagnosing pager behavior and measuring per-key latency with `--replay`.
