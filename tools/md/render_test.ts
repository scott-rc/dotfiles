import { assertEquals } from "@std/assert";
import { Lexer } from "marked";
import { renderTokens } from "./render.ts";
import { stripAnsi } from "./wrap.ts";

const WIDTH = 60;
const opts = { width: WIDTH };

function render(md: string): string {
  const tokens = new Lexer().lex(md);
  return renderTokens(tokens, opts);
}

function renderPlain(md: string): string {
  return stripAnsi(render(md));
}

// Headings

Deno.test("h1 is uppercased with underline", () => {
  const result = renderPlain("# Hello");
  const lines = result.split("\n");
  assertEquals(lines[0], "HELLO");
  assertEquals(lines[1][0], "═");
});

Deno.test("h2 has underline", () => {
  const result = renderPlain("## Section");
  const lines = result.split("\n");
  assertEquals(lines[0], "Section");
  assertEquals(lines[1][0], "─");
});

Deno.test("h3 renders without underline", () => {
  const result = renderPlain("### Sub");
  assertEquals(result, "Sub");
});

Deno.test("h4-h6 render without underline", () => {
  assertEquals(renderPlain("#### H4").trim(), "H4");
  assertEquals(renderPlain("##### H5").trim(), "H5");
  assertEquals(renderPlain("###### H6").trim(), "H6");
});

// Inline formatting

Deno.test("bold text has ANSI bold codes", () => {
  const result = render("**bold**");
  assertEquals(stripAnsi(result), "bold");
  // Should contain ANSI codes (more chars than visible)
  assertEquals(result.length > "bold".length, true);
});

Deno.test("italic text has ANSI italic codes", () => {
  const result = render("*italic*");
  assertEquals(stripAnsi(result), "italic");
  assertEquals(result.length > "italic".length, true);
});

Deno.test("inline code is padded", () => {
  const result = renderPlain("use `foo` here");
  assertEquals(result.includes(" foo "), true);
});

// Code blocks

Deno.test("code block has box borders", () => {
  const result = renderPlain("```\nhello\n```");
  assertEquals(result.includes("┌"), true);
  assertEquals(result.includes("└"), true);
  assertEquals(result.includes("│"), true);
});

Deno.test("code block shows language label", () => {
  const result = renderPlain("```typescript\nconst x = 1;\n```");
  assertEquals(result.includes("typescript"), true);
});

// Lists

Deno.test("unordered list uses bullet", () => {
  const result = renderPlain("- one\n- two\n- three");
  assertEquals(result.includes("•"), true);
  assertEquals(result.includes("one"), true);
  assertEquals(result.includes("two"), true);
});

Deno.test("ordered list uses numbers", () => {
  const result = renderPlain("1. first\n2. second");
  assertEquals(result.includes("1."), true);
  assertEquals(result.includes("2."), true);
});

Deno.test("nested lists indent", () => {
  const result = renderPlain("- outer\n  - inner");
  const lines = result.split("\n");
  const innerLine = lines.find((l) => l.includes("inner"))!;
  // Inner should be more indented than outer
  assertEquals(innerLine.search(/\S/) > 0, true);
});

// Blockquote

Deno.test("blockquote has border prefix", () => {
  const result = renderPlain("> quoted text");
  assertEquals(result.includes("│"), true);
  assertEquals(result.includes("quoted text"), true);
});

// Links

Deno.test("link shows text and URL", () => {
  const result = renderPlain("[example](https://example.com)");
  assertEquals(result.includes("example"), true);
  assertEquals(result.includes("https://example.com"), true);
});

// Horizontal rule

Deno.test("hr renders full-width line", () => {
  const result = renderPlain("---");
  assertEquals(result, "─".repeat(WIDTH));
});

// Word wrapping

Deno.test("paragraphs wrap to width", () => {
  const long = "word ".repeat(20).trim();
  const result = renderPlain(long);
  for (const line of result.split("\n")) {
    assertEquals(line.length <= WIDTH, true);
  }
});

// Integration: mixed document

Deno.test("renders mixed document", () => {
  const md = `# Title

Some **bold** and *italic* text.

## Code Example

\`\`\`js
const x = 1;
\`\`\`

- item one
- item two

> a quote

---

[link](https://example.com)
`;

  const result = renderPlain(md);
  assertEquals(result.includes("TITLE"), true);
  assertEquals(result.includes("bold"), true);
  assertEquals(result.includes("italic"), true);
  assertEquals(result.includes("const x = 1;"), true);
  assertEquals(result.includes("•"), true);
  assertEquals(result.includes("│"), true);
  assertEquals(result.includes("─"), true);
  assertEquals(result.includes("link"), true);
});
