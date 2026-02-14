import { assertEquals } from "@std/assert";
import {
  blockquoteText,
  codeLanguage,
  codeSpan,
  emStyle,
  frontmatterKey,
  frontmatterValue,
  h1,
  h2,
  h3,
  h4,
  h5,
  h6,
  hrStyle,
  linkText,
  linkUrl,
  listMarker,
  marker,
  strongStyle,
} from "./style.ts";
import { stripAnsi } from "./wrap.ts";

// Load palette for reference color values
const palette = JSON.parse(
  await Deno.readTextFile(
    new URL("./fixtures/style/palette.json", import.meta.url),
  ),
);

function rgb24Sequence(hex: number): string {
  const r = (hex >> 16) & 0xff;
  const g = (hex >> 8) & 0xff;
  const b = hex & 0xff;
  return `\x1b[38;2;${r};${g};${b}m`;
}

const HEADING_BLUE = parseInt(palette.HEADING_BLUE, 16);
const FOREGROUND = parseInt(palette.FOREGROUND, 16);
const CODE_ORANGE = parseInt(palette.CODE_ORANGE, 16);
const QUOTE_GREEN = parseInt(palette.QUOTE_GREEN, 16);
const LINK_BLUE = parseInt(palette.LINK_BLUE, 16);
const LIST_BLUE = parseInt(palette.LIST_BLUE, 16);
const COMMENT_GRAY = parseInt(palette.COMMENT_GRAY, 16);

// ── Heading styles ───────────────────────────────────────

Deno.test("h1 produces bold + HEADING_BLUE + uppercase", () => {
  const result = h1("hello");
  assertEquals(result.includes(rgb24Sequence(HEADING_BLUE)), true);
  assertEquals(result.includes("\x1b[1m"), true); // bold
  assertEquals(stripAnsi(result), "HELLO");
});

Deno.test("h1 uppercases text while preserving ANSI codes", () => {
  const inner = `\x1b[31mred\x1b[0m text`;
  const result = h1(inner);
  assertEquals(stripAnsi(result), "RED TEXT");
});

Deno.test("h2 produces bold + HEADING_BLUE, no uppercase", () => {
  const result = h2("hello");
  assertEquals(result.includes(rgb24Sequence(HEADING_BLUE)), true);
  assertEquals(result.includes("\x1b[1m"), true);
  assertEquals(stripAnsi(result), "hello");
});

Deno.test("h3 produces bold + HEADING_BLUE", () => {
  const result = h3("sub");
  assertEquals(result.includes(rgb24Sequence(HEADING_BLUE)), true);
  assertEquals(result.includes("\x1b[1m"), true);
});

Deno.test("h4 produces HEADING_BLUE without bold", () => {
  const result = h4("sub");
  assertEquals(result.includes(rgb24Sequence(HEADING_BLUE)), true);
  assertEquals(result.includes("\x1b[1m"), false);
});

Deno.test("h5 produces HEADING_BLUE without bold", () => {
  const result = h5("sub");
  assertEquals(result.includes(rgb24Sequence(HEADING_BLUE)), true);
  assertEquals(result.includes("\x1b[1m"), false);
});

Deno.test("h6 produces HEADING_BLUE without bold", () => {
  const result = h6("sub");
  assertEquals(result.includes(rgb24Sequence(HEADING_BLUE)), true);
  assertEquals(result.includes("\x1b[1m"), false);
});

// ── Marker style ─────────────────────────────────────────

Deno.test("marker produces COMMENT_GRAY", () => {
  const result = marker("#");
  assertEquals(result.includes(rgb24Sequence(COMMENT_GRAY)), true);
  assertEquals(stripAnsi(result), "#");
});

// ── List marker style ────────────────────────────────────

Deno.test("listMarker produces LIST_BLUE", () => {
  const result = listMarker("-");
  assertEquals(result.includes(rgb24Sequence(LIST_BLUE)), true);
  assertEquals(stripAnsi(result), "-");
});

// ── Inline styles ────────────────────────────────────────

Deno.test("strongStyle produces bold + FOREGROUND", () => {
  const result = strongStyle("bold");
  assertEquals(result.includes(rgb24Sequence(FOREGROUND)), true);
  assertEquals(result.includes("\x1b[1m"), true);
  assertEquals(stripAnsi(result), "bold");
});

Deno.test("emStyle produces italic + FOREGROUND", () => {
  const result = emStyle("italic");
  assertEquals(result.includes(rgb24Sequence(FOREGROUND)), true);
  assertEquals(result.includes("\x1b[3m"), true); // italic
  assertEquals(stripAnsi(result), "italic");
});

Deno.test("codeSpan produces gray backticks with orange content", () => {
  const result = codeSpan("foo");
  assertEquals(result.includes(rgb24Sequence(COMMENT_GRAY)), true);
  assertEquals(result.includes(rgb24Sequence(CODE_ORANGE)), true);
  assertEquals(stripAnsi(result), "`foo`");
});

// ── Code block language ──────────────────────────────────

Deno.test("codeLanguage produces italic + COMMENT_GRAY", () => {
  const result = codeLanguage("typescript");
  assertEquals(result.includes(rgb24Sequence(COMMENT_GRAY)), true);
  assertEquals(result.includes("\x1b[3m"), true); // italic
  assertEquals(stripAnsi(result), "typescript");
});

// ── Link styles ──────────────────────────────────────────

Deno.test("linkText produces underline + FOREGROUND", () => {
  const result = linkText("example");
  assertEquals(result.includes(rgb24Sequence(FOREGROUND)), true);
  assertEquals(result.includes("\x1b[4m"), true); // underline
  assertEquals(stripAnsi(result), "example");
});

Deno.test("linkUrl produces italic + underline + LINK_BLUE", () => {
  const result = linkUrl("https://example.com");
  assertEquals(result.includes(rgb24Sequence(LINK_BLUE)), true);
  assertEquals(result.includes("\x1b[3m"), true); // italic
  assertEquals(result.includes("\x1b[4m"), true); // underline
  assertEquals(stripAnsi(result), "https://example.com");
});

// ── Blockquote style ─────────────────────────────────────

Deno.test("blockquoteText produces QUOTE_GREEN", () => {
  const result = blockquoteText("quoted");
  assertEquals(result.includes(rgb24Sequence(QUOTE_GREEN)), true);
  assertEquals(stripAnsi(result), "quoted");
});

// ── HR style ─────────────────────────────────────────────

Deno.test("hrStyle produces COMMENT_GRAY", () => {
  const result = hrStyle("---");
  assertEquals(result.includes(rgb24Sequence(COMMENT_GRAY)), true);
  assertEquals(stripAnsi(result), "---");
});

// ── Frontmatter styles ───────────────────────────────────

Deno.test("frontmatterKey produces COMMENT_GRAY", () => {
  const result = frontmatterKey("title");
  assertEquals(result.includes(rgb24Sequence(COMMENT_GRAY)), true);
  assertEquals(stripAnsi(result), "title");
});

Deno.test("frontmatterValue produces FOREGROUND", () => {
  const result = frontmatterValue("My Doc");
  assertEquals(result.includes(rgb24Sequence(FOREGROUND)), true);
  assertEquals(stripAnsi(result), "My Doc");
});
