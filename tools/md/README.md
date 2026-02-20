# md

Terminal markdown renderer with color output, syntax highlighting, and an interactive pager.

Built by the parent `apply.sh` and symlinked to `~/.cargo/bin/md`.

## Usage

```
md [OPTIONS] [FILE]
```

`FILE` can be a file path, directory, or `-` for stdin. Piping also works: `cat README.md | md`.

### Options

| Flag | Description |
|------|-------------|
| `-w`, `--width <N>` | Set render width (default: terminal width, max 100) |
| `-p`, `--plain` | Disable Unicode decorations; use raw markdown syntax |
| `--no-color` | Disable ANSI color output |
| `--no-pager` | Disable the built-in pager |

When output fits in the terminal or stdout is not a TTY, the pager is skipped automatically.

## Features

- **Markdown rendering** — headings, bold, italic, strikethrough, code spans, code blocks, blockquotes, lists (ordered/unordered/nested/task lists), tables with alignment, horizontal rules, links, images, footnotes
- **Pretty mode** (default) — Unicode decorations: box-drawing borders on code blocks and tables, `•◦▪` bullets, `☑☐` task markers, `│` blockquotes, `═`/`─` heading underlines, full-width `─` horizontal rules, hidden inline delimiters (bold/italic/strikethrough/code) when color is on, clean link format. Disable with `--plain`
- **YAML frontmatter** — parsed and rendered as a key-value table above the body
- **Syntax highlighting** — fenced code blocks highlighted using the GitHub Dark theme (via syntect)
- **Word wrapping** — ANSI-aware with widow prevention (avoids leaving a single word on the last line)
- **Centering** — content is horizontally centered when the terminal is wider than the render width
- **Interactive pager** — alternate screen with search, clipboard, and editor integration. Status bar is hidden by default; appears only for search input and brief feedback messages
- **Directory browsing** — when given a directory, uses `find` + `fzf` (via `$SHELL`) to pick a `.md`/`.mdx` file, then renders it

## Pager Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Scroll down one line |
| `k` / `Up` | Scroll up one line |
| `d` / `Space` / `Ctrl-D` / `PageDown` | Scroll down half page |
| `u` / `Ctrl-U` / `PageUp` | Scroll up half page |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |

### Search

| Key | Action |
|-----|--------|
| `/` | Enter search mode |
| `n` | Next match |
| `N` | Previous match |
| `Enter` | Execute search |
| `Escape` / `Ctrl-C` | Cancel search |

Search input supports `Left`/`Right`, `Alt-Left`/`Alt-Right` (word jump), `Backspace`, `Alt-Backspace` (delete word), and `Ctrl-U` (delete to start).

### Clipboard & Editor

| Key | Action |
|-----|--------|
| `c` | Copy path to clipboard |
| `C` | Copy absolute path to clipboard |
| `y` | Copy raw markdown to clipboard |
| `e` | Open in `$EDITOR` (jumps to approximate line) |
| `v` | Open in `$EDITOR` read-only (jumps to approximate line) |

### Display

| Key | Action |
|-----|--------|
| `p` | Toggle plain/pretty mode |

### Help

| Key | Action |
|-----|--------|
| `?` | Show help overlay |

### Quit

| Key | Action |
|-----|--------|
| `q` / `Ctrl-C` | Quit pager |

## Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | CLI argument parsing (clap), input routing (file/stdin/directory), width calculation, centering |
| `render.rs` | Markdown-to-styled-text conversion using pulldown-cmark; handles all block and inline elements |
| `style.rs` | `Style` struct with color/pretty/plain-text formatting methods; GitHub Dark palette constants |
| `wrap.rs` | ANSI-aware word wrapping, `strip_ansi()`, `visible_length()`, `split_ansi()`, display wrapping |
| `pager.rs` | Interactive pager with alternate screen, search highlighting, hidden-by-default status bar, clipboard, editor launch |
| `browse.rs` | Directory browsing via `find` piped to `fzf`, spawned through `$SHELL` |
| `frontmatter.rs` | YAML frontmatter parser (extracts `---` delimited block into `IndexMap`) |
| `highlight.rs` | Syntax highlighting for fenced code blocks using syntect with a bundled GitHub Dark `.tmTheme` |

## Testing

```bash
cargo test
```

Three fixture systems:

- **Rendering** — `.md` + `.expected.txt` pairs in `fixtures/rendering/`, registered via `rendering_fixture!` / `frontmatter_fixture!` macros in `render.rs` tests. Width 60, no color.
- **Pretty** — `.md` + `.expected.txt` pairs in `fixtures/pretty/`, registered via `pretty_fixture!` macro. Width 60, no color, pretty mode enabled.
- **JSON** — per-module fixtures in `fixtures/{module}/` (e.g., `fixtures/pager/`), loaded via `include_str!()` with serde deserialization.
- **Integration** — `tests/integration.rs` spawns the binary via `CARGO_BIN_EXE_md` and uses the `run_md()` helper.

## Benchmarking

### Microbenchmarks

```bash
cargo bench
```

Criterion benchmarks in `benches/bench.rs` cover the render pipeline, word wrapping, ANSI stripping, and syntax highlighting. Results are stored in `target/criterion/` with HTML reports.

Compare against a baseline after making changes:

```bash
cargo bench -- --save-baseline before
# ... make changes ...
cargo bench -- --baseline before
```

### End-to-end timing

```bash
hyperfine --warmup 3 'md --no-pager ../../README.md'
```

Compare plain vs pretty:

```bash
hyperfine --warmup 3 \
  'md --no-pager --no-color --plain ../../README.md' \
  'md --no-pager ../../README.md'
```

### Profiling

Generate a flamegraph with [samply](https://github.com/mstange/samply):

```bash
cargo build --release
samply record ./target/release/md --no-pager --no-color ../../README.md
```

This opens the Firefox Profiler with a call tree and flame chart. Look for hot functions in `wrap.rs` (`strip_ansi`, `visible_length`, `word_wrap`) and `highlight.rs` (`highlight_code`).
