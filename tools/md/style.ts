import {
  blue,
  bold,
  cyan,
  dim,
  gray,
  italic,
  setColorEnabled,
  stripAnsiCode,
  underline,
  white,
} from "@std/fmt/colors";

export { setColorEnabled, stripAnsiCode };

// Headings
// deno-lint-ignore no-control-regex
const ANSI_RE = /\x1b\[[0-9;]*m/g;
const ansiUpperCase = (s: string) =>
  s.replace(ANSI_RE, "\0$&\0").split("\0").map((part) =>
    part.startsWith("\x1b") ? part : part.toUpperCase()
  ).join("");

export const h1 = (s: string) => bold(blue(ansiUpperCase(s)));
export const h2 = (s: string) => bold(blue(s));
export const h3 = (s: string) => bold(s);
export const h4 = (s: string) => white(s);
export const h5 = (s: string) => gray(s);
export const h6 = (s: string) => dim(s);

// Syntax markers (#, >, -, **, *, ```, [, ], (, ))
export const marker = gray;

// Inline
export const strongStyle = bold;
export const emStyle = italic;
export const codeSpan = (s: string) => `\`${s}\``;

// Code block
export const codeLanguage = (s: string) => italic(gray(s));

// Links
export const linkText = (s: string) => bold(blue(s));
export const linkUrl = (s: string) => cyan(underline(s));

// Blockquote
export const blockquoteText = italic;

// Horizontal rule
export const hrStyle = gray;
