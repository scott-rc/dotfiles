import {
  bold,
  italic,
  rgb24,
  setColorEnabled,
  stripAnsiCode,
  underline,
} from "@std/fmt/colors";

export { setColorEnabled, stripAnsiCode };

// GitHub Dark Default palette (from projekt0n/github-nvim-theme, github_dark_default)
const HEADING_BLUE = 0x79c0ff; // @markup.heading → Title → syntax.constant
const FOREGROUND = 0xe6edf3; // base foreground
const CODE_ORANGE = 0xffa657; // @markup.raw
const QUOTE_GREEN = 0x7ee787; // @markup.quote
const LINK_BLUE = 0x79c0ff; // @markup.link.uri
const LIST_BLUE = 0x79c0ff; // @markup.list
const COMMENT_GRAY = 0x8b949e; // comment

// Headings
// deno-lint-ignore no-control-regex
const ANSI_RE = /\x1b\[[0-9;]*m/g;
const ansiUpperCase = (s: string) =>
  s.replace(ANSI_RE, "\0$&\0").split("\0").map((part) =>
    part.startsWith("\x1b") ? part : part.toUpperCase()
  ).join("");

export const h1 = (s: string) => bold(rgb24(ansiUpperCase(s), HEADING_BLUE));
export const h2 = (s: string) => bold(rgb24(s, HEADING_BLUE));
export const h3 = (s: string) => bold(rgb24(s, HEADING_BLUE));
export const h4 = (s: string) => rgb24(s, HEADING_BLUE);
export const h5 = (s: string) => rgb24(s, HEADING_BLUE);
export const h6 = (s: string) => rgb24(s, HEADING_BLUE);

// Syntax markers (#, >, **, *, ```, [, ], (, ))
export const marker = (s: string) => rgb24(s, COMMENT_GRAY);

// List markers (-, 1., 2., etc.)
export const listMarker = (s: string) => rgb24(s, LIST_BLUE);

// Inline
export const strongStyle = (s: string) => bold(rgb24(s, FOREGROUND));
export const emStyle = (s: string) => italic(rgb24(s, FOREGROUND));
export const codeSpan = (s: string) =>
  rgb24("`", COMMENT_GRAY) + rgb24(s, CODE_ORANGE) + rgb24("`", COMMENT_GRAY);

// Code block
export const codeLanguage = (s: string) => italic(rgb24(s, COMMENT_GRAY));

// Links
export const linkText = (s: string) => underline(rgb24(s, FOREGROUND));
export const linkUrl = (s: string) => italic(underline(rgb24(s, LINK_BLUE)));

// Blockquote
export const blockquoteText = (s: string) => rgb24(s, QUOTE_GREEN);

// Horizontal rule
export const hrStyle = (s: string) => rgb24(s, COMMENT_GRAY);

// Frontmatter
export const frontmatterKey = (s: string) => rgb24(s, COMMENT_GRAY);
export const frontmatterValue = (s: string) => rgb24(s, FOREGROUND);
