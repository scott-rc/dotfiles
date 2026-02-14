import { assertEquals } from "@std/assert";
import { Lexer } from "marked";
import { renderFrontmatter, renderTokens } from "./render.ts";
import { renderMarkdown } from "./mod.ts";
import { stripAnsi } from "./wrap.ts";

const WIDTH = 60;
const opts = { width: WIDTH };
const FIXTURE_DIR = new URL("./fixtures/rendering/", import.meta.url);

async function render(md: string): Promise<string> {
  const tokens = new Lexer().lex(md);
  return await renderTokens(tokens, opts);
}

async function renderPlain(md: string): Promise<string> {
  return stripAnsi(await render(md));
}

async function renderMd(md: string): Promise<string> {
  return stripAnsi(await renderMarkdown(md, opts));
}

// ── Rendering fixtures (ANSI-stripped) ───────────────────

const renderingFixtures = [
  "heading-h1",
  "heading-h2",
  "bold",
  "italic",
  "inline-code",
  "code-block-plain",
  "code-block-lang",
  "unordered-list",
  "ordered-list",
  "nested-list",
  "blockquote",
  "link",
  "hr",
  "paragraph-wrap",
  "mixed-document",
];

for (const name of renderingFixtures) {
  Deno.test(`rendering fixture: ${name}`, async () => {
    const input = await Deno.readTextFile(new URL(`${name}.md`, FIXTURE_DIR));
    const expected = await Deno.readTextFile(
      new URL(`${name}.expected.txt`, FIXTURE_DIR),
    );
    const result = await renderPlain(input);
    assertEquals(result, expected);
  });
}

// Frontmatter fixtures use renderMarkdown (via mod.ts)
const frontmatterFixtures = [
  "frontmatter-basic",
  "frontmatter-arrays",
  "frontmatter-empty",
  "frontmatter-malformed",
  "bare-hr-not-frontmatter",
];

for (const name of frontmatterFixtures) {
  Deno.test(`rendering fixture: ${name}`, async () => {
    const input = await Deno.readTextFile(new URL(`${name}.md`, FIXTURE_DIR));
    const expected = await Deno.readTextFile(
      new URL(`${name}.expected.txt`, FIXTURE_DIR),
    );
    const result = await renderMd(input);
    assertEquals(result, expected);
  });
}

// ── Heading-specific tests ───────────────────────────────

Deno.test("h1 is uppercased with # prefix", async () => {
  assertEquals(await renderPlain("# Hello"), "# HELLO");
});

Deno.test("h4-h6 have # prefixes", async () => {
  assertEquals(await renderPlain("#### H4"), "#### H4");
  assertEquals(await renderPlain("##### H5"), "##### H5");
  assertEquals(await renderPlain("###### H6"), "###### H6");
});

// ── Paragraphs wrap to width ─────────────────────────────

Deno.test("paragraphs wrap to width", async () => {
  const long = "word ".repeat(20).trim();
  const result = await renderPlain(long);
  for (const line of result.split("\n")) {
    assertEquals(line.length <= WIDTH, true);
  }
});

// ── Frontmatter unit tests ───────────────────────────────

Deno.test("renderFrontmatter formats key-value pairs", () => {
  const result = stripAnsi(
    renderFrontmatter({ title: "My Doc", date: "2024-01-01" }),
  );
  assertEquals(result.includes("title"), true);
  assertEquals(result.includes("My Doc"), true);
});

Deno.test("renderFrontmatter aligns keys", () => {
  const result = stripAnsi(renderFrontmatter({ ab: "x", abcd: "y" }));
  const lines = result.split("\n");
  assertEquals(lines[0].startsWith("ab  "), true);
  assertEquals(lines[1].startsWith("abcd"), true);
});

Deno.test("renderFrontmatter joins arrays with commas", () => {
  const result = stripAnsi(
    renderFrontmatter({ tags: ["one", "two", "three"] }),
  );
  assertEquals(result.includes("one, two, three"), true);
});

Deno.test("renderFrontmatter returns empty string for empty attrs", () => {
  assertEquals(renderFrontmatter({}), "");
});

Deno.test("renderFrontmatter wraps long values to width", () => {
  const long = "word ".repeat(20).trim();
  const result = stripAnsi(renderFrontmatter({ description: long }, 40));
  const lines = result.split("\n");
  assertEquals(lines.length > 1, true);
  for (const line of lines) {
    assertEquals(line.length <= 40, true);
  }
  for (const line of lines.slice(1)) {
    assertEquals(line.startsWith("             "), true);
  }
});

// ── Frontmatter integration ──────────────────────────────

Deno.test("frontmatter is extracted and rendered at top", async () => {
  const md = `---
title: My Doc
date: 2024-01-01
---

# Hello`;
  const result = await renderMd(md);
  const titleIdx = result.indexOf("My Doc");
  const headingIdx = result.indexOf("# HELLO");
  assertEquals(titleIdx < headingIdx, true);
});

Deno.test("no frontmatter renders normally", async () => {
  const result = await renderMd("# Hello\n\nSome text.");
  assertEquals(result, "# HELLO\n\nSome text.");
});

Deno.test("malformed YAML frontmatter does not crash", async () => {
  const md = `---
: [[[invalid
---

# Hello`;
  const result = await renderMd(md);
  assertEquals(typeof result, "string");
});

// ── ANSI rendering assertions ────────────────────────────
// Verify specific ANSI sequences are present in styled output.

function rgb24Seq(hex: number): string {
  const r = (hex >> 16) & 0xff;
  const g = (hex >> 8) & 0xff;
  const b = hex & 0xff;
  return `\x1b[38;2;${r};${g};${b}m`;
}

const HEADING_BLUE = 0x79c0ff;
const COMMENT_GRAY = 0x8b949e;
const CODE_ORANGE = 0xffa657;
const QUOTE_GREEN = 0x7ee787;

Deno.test("ANSI: h1 contains bold and HEADING_BLUE", async () => {
  const result = await render("# Hello");
  assertEquals(result.includes("\x1b[1m"), true);
  assertEquals(result.includes(rgb24Seq(HEADING_BLUE)), true);
});

Deno.test("ANSI: h2 contains bold and HEADING_BLUE", async () => {
  const result = await render("## Hello");
  assertEquals(result.includes("\x1b[1m"), true);
  assertEquals(result.includes(rgb24Seq(HEADING_BLUE)), true);
});

Deno.test("ANSI: code fence markers contain COMMENT_GRAY", async () => {
  const result = await render("```\nhello\n```");
  assertEquals(result.includes(rgb24Seq(COMMENT_GRAY)), true);
});

Deno.test("ANSI: inline code contains CODE_ORANGE", async () => {
  const result = await render("use `foo` here");
  assertEquals(result.includes(rgb24Seq(CODE_ORANGE)), true);
});

Deno.test("ANSI: inline code backticks are COMMENT_GRAY", async () => {
  const result = await render("use `foo` here");
  assertEquals(result.includes(rgb24Seq(COMMENT_GRAY)), true);
});

Deno.test("ANSI: bold markers are COMMENT_GRAY", async () => {
  const result = await render("**bold**");
  assertEquals(result.includes(rgb24Seq(COMMENT_GRAY)), true);
});

Deno.test("ANSI: blockquote text is QUOTE_GREEN", async () => {
  const result = await render("> quoted text");
  assertEquals(result.includes(rgb24Seq(QUOTE_GREEN)), true);
});

Deno.test("ANSI: hr is COMMENT_GRAY", async () => {
  const result = await render("---");
  assertEquals(result.includes(rgb24Seq(COMMENT_GRAY)), true);
});

Deno.test("ANSI: list marker is HEADING_BLUE", async () => {
  const result = await render("- item");
  // LIST_BLUE === HEADING_BLUE (0x79c0ff)
  assertEquals(result.includes(rgb24Seq(HEADING_BLUE)), true);
});
