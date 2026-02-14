import {
  bold,
  cyan,
  dim,
  green,
  italic,
  magenta,
  setColorEnabled,
  stripAnsiCode,
  underline,
  yellow,
} from "@std/fmt/colors";

export { setColorEnabled, stripAnsiCode };

// Headings
// deno-lint-ignore no-control-regex
const ANSI_RE = /\x1b\[[0-9;]*m/g;
const ansiUpperCase = (s: string) =>
  s.replace(ANSI_RE, "\0$&\0").split("\0").map((part) =>
    part.startsWith("\x1b") ? part : part.toUpperCase()
  ).join("");

export const h1 = (s: string) => bold(magenta(ansiUpperCase(s)));
export const h2 = (s: string) => bold(cyan(s));
export const h3 = (s: string) => bold(yellow(s));
export const h4 = (s: string) => bold(green(s));
export const h5 = (s: string) => bold(magenta(s));
export const h6 = (s: string) => bold(dim(s));

// Inline
export const strongStyle = bold;
export const emStyle = italic;
export const codeSpan = (s: string) => dim(` ${s} `);

// Code block
export const codeBorder = dim;
export const codeLanguage = (s: string) => dim(italic(s));

// Links
export const linkText = bold;
export const linkUrl = (s: string) => cyan(underline(s));

// Blockquote
export const blockquoteBorder = (s: string) => dim(s);
export const blockquoteText = (s: string) => dim(italic(s));

// Lists
export const bullet = dim;

// Horizontal rule
export const hrStyle = dim;

// Heading underlines
export const h1Underline = "═";
export const h2Underline = "─";
