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
| `--no-color` | Disable ANSI color output |
| `--no-pager` | Disable the built-in pager |

When output fits in the terminal or stdout is not a TTY, the pager is skipped automatically.

## Features

- **Markdown rendering** — headings, bold, italic, code spans, code blocks, blockquotes, lists (ordered/unordered/nested), tables with alignment, horizontal rules, links, images
- **YAML frontmatter** — parsed and rendered as a key-value table above the body
- **Syntax highlighting** — fenced code blocks highlighted using the GitHub Dark theme (via syntect)
- **Word wrapping** — ANSI-aware with widow prevention (avoids leaving a single word on the last line)
- **Centering** — content is horizontally centered when the terminal is wider than the render width
- **Interactive pager** — alternate screen with search, clipboard, and editor integration
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
| `c` | Copy filename to clipboard |
| `C` | Copy absolute path to clipboard |
| `y` | Copy raw markdown to clipboard |
| `v` | Open in `$EDITOR` (read-only for vim/nvim, jumps to approximate line) |

### Quit

| Key | Action |
|-----|--------|
| `q` / `Ctrl-C` | Quit pager |

## Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | CLI argument parsing (clap), input routing (file/stdin/directory), width calculation, centering |
| `render.rs` | Markdown-to-styled-text conversion using pulldown-cmark; handles all block and inline elements |
| `style.rs` | `Style` struct with color/plain-text formatting methods; GitHub Dark palette constants |
| `wrap.rs` | ANSI-aware word wrapping, `strip_ansi()`, `visible_length()`, `split_ansi()`, display wrapping |
| `pager.rs` | Interactive pager with alternate screen, search highlighting, status bar, clipboard, editor launch |
| `browse.rs` | Directory browsing via `find` piped to `fzf`, spawned through `$SHELL` |
| `frontmatter.rs` | YAML frontmatter parser (extracts `---` delimited block into `IndexMap`) |
| `highlight.rs` | Syntax highlighting for fenced code blocks using syntect with a bundled GitHub Dark `.tmTheme` |

## Testing

```bash
cargo test
```

Three fixture systems:

- **Rendering** — `.md` + `.expected.txt` pairs in `fixtures/rendering/`, registered via `rendering_fixture!` / `frontmatter_fixture!` macros in `render.rs` tests. Width 60, no color.
- **JSON** — per-module fixtures in `fixtures/{module}/` (e.g., `fixtures/pager/`), loaded via `include_str!()` with serde deserialization.
- **Integration** — `tests/integration.rs` spawns the binary via `CARGO_BIN_EXE_md` and uses the `run_md()` helper.
