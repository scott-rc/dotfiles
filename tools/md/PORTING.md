# Porting Guide

This document covers everything needed to port the `md` terminal markdown renderer to another language.

## Module Dependency Graph

```
main.ts → mod.ts → render.ts → { style.ts, highlight.ts, wrap.ts }
main.ts → pager.ts → wrap.ts
main.ts → browse.ts
```

`wrap.ts` and `style.ts` are leaf modules with no internal dependencies. `browse.ts` is standalone. `pager.ts` depends only on `wrap.ts`.

## Recommended Port Order

1. **`wrap.ts`** — Pure functions, no dependencies, extensive test fixtures. Start here.
2. **`style.ts`** — Pure ANSI generation. Use `fixtures/style/palette.json` as reference.
3. **`render.ts`** — Core pipeline. Depends on wrap + style + highlight. Token-by-token rendering.
4. **`highlight.ts`** — Thin wrapper around a syntax highlighting library.
5. **`mod.ts`** — Thin orchestrator: frontmatter extraction + render. Small file.
6. **`main.ts`** — CLI entry point: I/O, arg parsing, centering, pager launch.
7. **`browse.ts`** — Shell-out utility for directory browsing with fzf.
8. **`pager.ts`** — Most complex module. Port utility functions first (they have fixtures), main event loop last.

## Library Equivalents

| Deno dep | Purpose | Rust | Go |
|----------|---------|------|----|
| `marked` | Markdown lexer → token stream | `pulldown-cmark` / `comrak` | `goldmark` |
| `shiki` | Syntax highlighting (TextMate grammars) | `syntect` | `chroma` |
| `@std/fmt/colors` | ANSI escape code generation | `owo-colors` / `colored` | `fatih/color` |
| `@std/front-matter` | YAML frontmatter detection + extraction | `serde_yaml` + manual `---` split | `go-yaml` + manual `---` split |
| `@std/cli` | Argument parsing | `clap` | `flag` / `cobra` |

## Token Type Mapping

The renderer consumes `marked`'s token stream. Each token type maps to a specific output format:

| Token type | Handler | Output format |
|------------|---------|---------------|
| `heading` | `renderHeading` | `# PREFIX` + styled text (h1 is uppercased) |
| `paragraph` | `renderParagraph` | Word-wrapped inline content |
| `code` | `renderCodeBlock` | `` ``` `` fence + highlighted body + `` ``` `` |
| `blockquote` | `renderBlockquote` | `> ` prefix on each line, green text, recursive |
| `list` | `renderList` | `- ` or `N. ` markers, recursive nesting |
| `hr` | `renderHr` | Gray `---` |
| `html` | (treated as paragraph) | Word-wrapped raw text |
| `space` | (skipped) | Returns null |

**Inline tokens** (inside paragraphs, headings, list items):

| Token type | Output |
|------------|--------|
| `text` | Literal text (may contain nested tokens) |
| `strong` | `**text**` with gray markers, bold+foreground text |
| `em` | `*text*` with gray markers, italic+foreground text |
| `codespan` | `` `text` `` with gray backticks, orange content |
| `link` | `[text](url)` with gray brackets, underlined text, italic+underlined URL |
| `br` | Newline character |
| `escape` | Literal escaped character |

**Important:** Different markdown parsers produce different token structures. `marked` emits a flat token array where each token may have a `.tokens` sub-array for inline content. Other parsers (pulldown-cmark, goldmark) may use event streams or nested ASTs. The rendering logic must be adapted accordingly.

### Top-level joining

Rendered top-level tokens are joined with `\n\n` (double newline). Null results (from `space` tokens) are filtered out before joining.

## Behavioral Edge Cases

### Widow prevention (wrap.ts)

The word wrapper avoids leaving a single word on the last line of a wrapped paragraph. After greedy wrapping, if the last line contains only one word, it retries with narrower widths (width-1 down to width-15) looking for a layout that puts at least two words on the last line. This is a visual quality feature — the fallback is to keep the greedy result.

### Backtick-aware line breaking (wrap.ts)

When wrapping, the algorithm checks for unpaired backticks at the end of a line. If a line ends with an odd number of visible backticks (indicating an opening code span), the backtick and following word are moved to the next line. This prevents code spans from being split across lines in a confusing way.

### h1 ANSI-aware uppercasing (style.ts)

The h1 style uppercases visible text while preserving ANSI escape codes. It works by:
1. Using a regex to insert null bytes around ANSI codes
2. Splitting on null bytes
3. Uppercasing non-ANSI parts
4. Rejoining

### Code blocks are NOT word-wrapped

Code block content is rendered with syntax highlighting but never word-wrapped — it preserves the original formatting. Only the fence markers (```` ``` ````) and optional language label are styled.

### List content width calculation

For lists, the available width for content is: `width - indent - markerWidth`, where indent is `"  ".repeat(depth)` and markerWidth is the visible length of the styled marker (e.g., `"- "` = 2, `"1. "` = 3). Continuation lines are indented to align with the content start, not the marker.

### Frontmatter key alignment

All frontmatter keys are right-padded to match the longest key length. Values are separated from keys by two spaces. Long values are word-wrapped with continuation lines indented to align with the value start position.

### Scroll position mapping (pager.ts)

When the terminal is resized, the pager re-renders content at the new width, which changes the line count. The scroll position is mapped proportionally: `newTop = round((oldTop / (oldCount - 1)) * (newCount - 1))`. Edge case: if either count is 1, position resets to 0.

### Search highlighting (pager.ts)

Search is case-insensitive and operates on ANSI-stripped text. Highlighting uses reverse video (`ESC[7m` / `ESC[27m`) injected at the correct positions in the original ANSI-containing string, using a position map from visible character indices to original string indices.

### Status bar layout (pager.ts)

The status bar uses a left/right split:
- **Left:** filename (normal), `/query (N/M)` (search active), or message
- **Right:** dimmed line range + position (TOP/END/percent)
- **Narrow terminal:** left side truncates to preserve right side
- **Width contract:** visible width always equals terminal columns exactly

## Color Palette

See `fixtures/style/palette.json` for hex values. The palette is GitHub Dark Default (from `projekt0n/github-nvim-theme`):

| Element | Color | Hex |
|---------|-------|-----|
| Headings, links, lists | Blue | `#79c0ff` |
| Base text | Foreground | `#e6edf3` |
| Inline code content | Orange | `#ffa657` |
| Blockquote text | Green | `#7ee787` |
| Syntax markers, comments | Gray | `#8b949e` |

## Test Fixtures

Language-agnostic test fixtures are in `fixtures/`:

- `fixtures/rendering/*.md` + `*.expected.txt` — Input→output pairs for full rendering (ANSI-stripped, width 60)
- `fixtures/wrapping/*.json` — Word wrap, stripAnsi, visibleLength test cases
- `fixtures/pager/*.json` — parseKey, truncateLine, highlightSearch, findMatches, mapScrollPosition, findNearestMatch, formatStatusBar, wordBoundary, handleSearchKey, mapToSourceLine
- `fixtures/browse/*.json` — shellQuote, buildFindCmd, buildPickCmd, parseSelection, shouldPage
- `fixtures/style/palette.json` — Color hex values

### Running fixtures against your port

The `compat/` directory contains shell scripts that test any `md` binary:

```bash
# Rendering comparison (reads fixtures, diffs output)
./compat/run_compat_tests.sh './your-binary'

# CLI behavior tests (flags, stdin, error handling)
./compat/cli_tests.sh './your-binary'
```

### Regenerating fixtures

If the Deno implementation changes:

```bash
deno task generate-fixtures   # Regenerate from current implementation
deno task verify-fixtures     # Verify fixtures match implementation
```

## CLI Interface

The CLI must support these flags for compat tests to pass:

| Flag | Behavior |
|------|----------|
| `--help` or `-h` | Print usage, exit 0 |
| `--width <n>` or `-w <n>` | Set render width (default: min(terminal, 80)) |
| `--no-color` | Disable all ANSI color output |
| `--no-pager` | Write to stdout instead of pager |
| `-` (positional) | Read from stdin |
| `<file>` (positional) | Read from file |
| `<directory>` (positional) | Browse directory with fzf |

Nonexistent file arguments should exit with code 1. Empty files should exit with code 0.
