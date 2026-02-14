# md

Terminal markdown renderer built with Deno and [marked](https://github.com/markedjs/marked). Renders markdown with color output using a GitHub Dark Default palette, syntax highlighting for code blocks via [shiki](https://shiki.style/) (TextMate grammars, `github-dark` theme), word wrapping, and a built-in pager.

## Usage

```bash
md <file>           # Render a markdown file
md <directory>      # Browse markdown files with fzf
md -                # Read from stdin
cat README.md | md  # Piped input
```

## Options

| Flag | Description |
|------|-------------|
| `-w, --width <n>` | Set output width (default: min(terminal, 80)) |
| `--no-color` | Disable color output |
| `--no-pager` | Disable built-in pager |

## Directory Browsing

When given a directory, `md` pipes a find command into a pick command and opens the selected file in the pager. Quitting the pager returns to the picker; quitting fzf exits. The shell used is `$SHELL` (falls back to `sh`).

| Variable | Default | Description |
|----------|---------|-------------|
| `MD_FIND_CMD` | `find {dir} -type f \( -name '*.md' -o -name '*.mdx' \)` | Command to find files. `{dir}` is replaced with the directory path; if absent, the directory is appended. |
| `MD_PICK_CMD` | `fzf` | Command to pick a file from stdin. Inherits `FZF_DEFAULT_OPTS` from the environment. |

The defaults use only POSIX `find` and `fzf`. The fish config (`conf.d/md.fish`) layers on `fd`, `fzf_files`, preview, and `--scheme=path` sorting via `MD_FIND_CMD` and `MD_PICK_CMD` environment variables.

## Frontmatter

YAML frontmatter (delimited by `---`) is automatically detected and rendered as a styled key-value block at the top of the output. Array values are displayed as comma-separated lists. Files without frontmatter render normally.

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
| `main.ts` | CLI entry point — arg parsing, stdin/file/directory handling, centering, pager launch |
| `browse.ts` | Directory browsing — command construction, shell-out via `$SHELL`, selection loop |
| `mod.ts` | Public API — `renderMarkdown()` with YAML frontmatter extraction via marked's lexer |
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
| `Left` / `Right` | Move cursor in search input |
| `Option+Left` / `Option+Right` | Move cursor by word in search input |
| `Option+Delete` | Delete word before cursor in search input |
| `Cmd+Delete` | Delete to beginning of search input |
| `c` | Copy relative file path to clipboard |
| `C` | Copy absolute file path to clipboard |
| `y` | Copy raw markdown source to clipboard |
| `v` | Open file in `$EDITOR` (default: nvim, readonly) |
| `q` / `Ctrl-C` | Quit |

## Status Bar

The bottom status bar uses a left/right split layout (like `less`). The filename appears on the left, with a dimmed line range and position indicator on the right. Position shows `TOP` at the beginning, `END` at the bottom, or a percentage when scrolled mid-document. During search, the left side shows the query with match count; in input mode, a block cursor follows the typed text. When the terminal is too narrow, the left side truncates to preserve position info.
