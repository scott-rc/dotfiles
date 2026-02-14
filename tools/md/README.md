# md

Terminal markdown renderer built with Deno and [marked](https://github.com/markedjs/marked). Renders markdown with color output using a GitHub Dark Default palette, syntax highlighting for code blocks via [shiki](https://shiki.style/) (TextMate grammars, `github-dark` theme), word wrapping, and a built-in pager.

## Usage

```bash
md <file>           # Render a markdown file
md -                # Read from stdin
cat README.md | md  # Piped input
```

## Options

| Flag | Description |
|------|-------------|
| `-w, --width <n>` | Set output width (default: min(terminal, 80)) |
| `--no-color` | Disable color output |
| `--no-pager` | Disable built-in pager |

## Install

```bash
deno task install
```

## Development

```bash
deno task dev       # Run with permissions
deno task test      # Run tests
```

## Architecture

| Module | Description |
|--------|-------------|
| `main.ts` | CLI entry point — arg parsing, stdin/file reading, centering, pager launch |
| `mod.ts` | Public API — `renderMarkdown()` using marked's lexer |
| `render.ts` | Token renderer — headings, paragraphs, code blocks, lists, blockquotes, inline styles |
| `highlight.ts` | Syntax highlighting for code blocks using shiki (`github-dark` theme) |
| `style.ts` | Color palette (GitHub Dark Default from github-nvim-theme) and ANSI formatting |
| `wrap.ts` | ANSI-aware word wrap with widow prevention |
| `pager.ts` | Built-in pager with alternate screen, search highlighting, resize handling, and left/right split status bar |

## Pager Keybindings

| Key | Action |
|-----|--------|
| `j` / `Down` / `Enter` | Scroll down one line |
| `k` / `Up` | Scroll up one line |
| `d` / `Space` / `Ctrl-D` / `Page Down` | Scroll down half page |
| `u` / `Ctrl-U` / `Page Up` | Scroll up half page |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |
| `/` | Search |
| `n` | Next match |
| `N` | Previous match |
| `c` | Copy relative file path to clipboard |
| `C` | Copy absolute file path to clipboard |
| `y` | Copy raw markdown source to clipboard |
| `v` | Open file in `$EDITOR` (default: nvim, readonly) |
| `q` / `Ctrl-C` | Quit |

## Status Bar

The bottom status bar uses a left/right split layout (like `less`). The filename appears on the left, with a dimmed line range and position indicator on the right. Position shows `TOP` at the beginning, `END` at the bottom, or a percentage when scrolled mid-document. During search, the left side shows the query with match count; in input mode, a block cursor follows the typed text. When the terminal is too narrow, the left side truncates to preserve position info.
