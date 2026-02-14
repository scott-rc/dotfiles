import { assertEquals } from "@std/assert";
import { Lexer } from "marked";
import { renderTokens } from "./render.ts";
import { stripAnsi } from "./wrap.ts";

const WIDTH = 60;
const opts = { width: WIDTH };

async function render(md: string): Promise<string> {
  const tokens = new Lexer().lex(md);
  return await renderTokens(tokens, opts);
}

async function renderPlain(md: string): Promise<string> {
  return stripAnsi(await render(md));
}

// Headings

Deno.test("h1 is uppercased with # prefix", async () => {
  const result = await renderPlain("# Hello");
  assertEquals(result, "# HELLO");
});

Deno.test("h2 has ## prefix", async () => {
  const result = await renderPlain("## Section");
  assertEquals(result, "## Section");
});

Deno.test("h3 has ### prefix", async () => {
  const result = await renderPlain("### Sub");
  assertEquals(result, "### Sub");
});

Deno.test("h4-h6 have # prefixes", async () => {
  assertEquals(await renderPlain("#### H4"), "#### H4");
  assertEquals(await renderPlain("##### H5"), "##### H5");
  assertEquals(await renderPlain("###### H6"), "###### H6");
});

// Inline formatting

Deno.test("bold text preserves ** markers", async () => {
  const result = await renderPlain("**bold**");
  assertEquals(result, "**bold**");
});

Deno.test("italic text preserves * markers", async () => {
  const result = await renderPlain("*italic*");
  assertEquals(result, "*italic*");
});

Deno.test("inline code has backticks", async () => {
  const result = await renderPlain("use `foo` here");
  assertEquals(result.includes("`foo`"), true);
});

// Code blocks

Deno.test("code block has ``` fences", async () => {
  const result = await renderPlain("```\nhello\n```");
  const lines = result.split("\n");
  assertEquals(lines[0], "```");
  assertEquals(lines[1], "hello");
  assertEquals(lines[2], "```");
});

Deno.test("code block shows language on opening fence", async () => {
  const result = await renderPlain("```typescript\nconst x = 1;\n```");
  const lines = result.split("\n");
  assertEquals(lines[0], "```typescript");
  assertEquals(lines[1], "const x = 1;");
  assertEquals(lines[2], "```");
});

// Lists

Deno.test("unordered list uses -", async () => {
  const result = await renderPlain("- one\n- two\n- three");
  assertEquals(result.includes("- one"), true);
  assertEquals(result.includes("- two"), true);
  assertEquals(result.includes("- three"), true);
});

Deno.test("ordered list uses numbers", async () => {
  const result = await renderPlain("1. first\n2. second");
  assertEquals(result.includes("1."), true);
  assertEquals(result.includes("2."), true);
});

Deno.test("nested lists indent", async () => {
  const result = await renderPlain("- outer\n  - inner");
  const lines = result.split("\n");
  const innerLine = lines.find((l) => l.includes("inner"))!;
  // Inner should be more indented than outer
  assertEquals(innerLine.search(/\S/) > 0, true);
});

// Blockquote

Deno.test("blockquote has > prefix", async () => {
  const result = await renderPlain("> quoted text");
  assertEquals(result.includes(">"), true);
  assertEquals(result.includes("quoted text"), true);
});

// Links

Deno.test("link preserves [text](url) format", async () => {
  const result = await renderPlain("[example](https://example.com)");
  assertEquals(result.includes("[example]"), true);
  assertEquals(result.includes("](https://example.com)"), true);
});

// Horizontal rule

Deno.test("hr renders as ---", async () => {
  const result = await renderPlain("---");
  assertEquals(result, "---");
});

// Word wrapping

Deno.test("paragraphs wrap to width", async () => {
  const long = "word ".repeat(20).trim();
  const result = await renderPlain(long);
  for (const line of result.split("\n")) {
    assertEquals(line.length <= WIDTH, true);
  }
});

// Integration: mixed document

Deno.test("renders mixed document", async () => {
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

  const result = await renderPlain(md);
  assertEquals(result.includes("# TITLE"), true);
  assertEquals(result.includes("**bold**"), true);
  assertEquals(result.includes("*italic*"), true);
  assertEquals(result.includes("const x = 1;"), true);
  assertEquals(result.includes("- item"), true);
  assertEquals(result.includes(">"), true);
  assertEquals(result.includes("---"), true);
  assertEquals(result.includes("[link]"), true);
});
