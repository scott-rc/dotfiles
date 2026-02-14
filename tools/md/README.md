# md

Terminal markdown renderer built with Deno and [marked](https://github.com/markedjs/marked). Renders markdown with color output using a GitHub Dark Default palette, word wrapping, and a built-in pager.

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
| `style.ts` | Color palette (GitHub Dark Default from github-nvim-theme) and ANSI formatting |
| `wrap.ts` | ANSI-aware word wrap with widow prevention |
| `pager.ts` | Built-in pager with alternate screen, search highlighting, and status bar |

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
| `q` / `Ctrl-C` | Quit |
