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

Deno.test("h1 is uppercased with # prefix", () => {
  const result = renderPlain("# Hello");
  assertEquals(result, "# HELLO");
});

Deno.test("h2 has ## prefix", () => {
  const result = renderPlain("## Section");
  assertEquals(result, "## Section");
});

Deno.test("h3 has ### prefix", () => {
  const result = renderPlain("### Sub");
  assertEquals(result, "### Sub");
});

Deno.test("h4-h6 have # prefixes", () => {
  assertEquals(renderPlain("#### H4"), "#### H4");
  assertEquals(renderPlain("##### H5"), "##### H5");
  assertEquals(renderPlain("###### H6"), "###### H6");
});

// Inline formatting

Deno.test("bold text preserves ** markers", () => {
  const result = renderPlain("**bold**");
  assertEquals(result, "**bold**");
});

Deno.test("italic text preserves * markers", () => {
  const result = renderPlain("*italic*");
  assertEquals(result, "*italic*");
});

Deno.test("inline code has backticks", () => {
  const result = renderPlain("use `foo` here");
  assertEquals(result.includes("`foo`"), true);
});

// Code blocks

Deno.test("code block has ``` fences", () => {
  const result = renderPlain("```\nhello\n```");
  const lines = result.split("\n");
  assertEquals(lines[0], "```");
  assertEquals(lines[1], "hello");
  assertEquals(lines[2], "```");
});

Deno.test("code block shows language on opening fence", () => {
  const result = renderPlain("```typescript\nconst x = 1;\n```");
  const lines = result.split("\n");
  assertEquals(lines[0], "```typescript");
  assertEquals(lines[1], "const x = 1;");
  assertEquals(lines[2], "```");
});

// Lists

Deno.test("unordered list uses -", () => {
  const result = renderPlain("- one\n- two\n- three");
  assertEquals(result.includes("- one"), true);
  assertEquals(result.includes("- two"), true);
  assertEquals(result.includes("- three"), true);
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

Deno.test("blockquote has > prefix", () => {
  const result = renderPlain("> quoted text");
  assertEquals(result.includes(">"), true);
  assertEquals(result.includes("quoted text"), true);
});

// Links

Deno.test("link preserves [text](url) format", () => {
  const result = renderPlain("[example](https://example.com)");
  assertEquals(result.includes("[example]"), true);
  assertEquals(result.includes("](https://example.com)"), true);
});

// Horizontal rule

Deno.test("hr renders as ---", () => {
  const result = renderPlain("---");
  assertEquals(result, "---");
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
  assertEquals(result.includes("# TITLE"), true);
  assertEquals(result.includes("**bold**"), true);
  assertEquals(result.includes("*italic*"), true);
  assertEquals(result.includes("const x = 1;"), true);
  assertEquals(result.includes("- item"), true);
  assertEquals(result.includes(">"), true);
  assertEquals(result.includes("---"), true);
  assertEquals(result.includes("[link]"), true);
});
