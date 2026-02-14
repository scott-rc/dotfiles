import { assertEquals } from "@std/assert";
import { highlightCode } from "./highlight.ts";
import { stripAnsi } from "./wrap.ts";

Deno.test("known language produces ANSI codes", async () => {
  const result = await highlightCode("const x = 1;", "typescript");
  // Should contain ANSI escape sequences
  assertEquals(result !== "const x = 1;", true);
  assertEquals(result.includes("\x1b["), true);
});

Deno.test("unknown language returns plain text", async () => {
  const result = await highlightCode("hello world", "not-a-language");
  assertEquals(result, "hello world");
});

Deno.test("no language returns plain text", async () => {
  const result = await highlightCode("hello world");
  assertEquals(result, "hello world");
});

Deno.test("stripping ANSI preserves original text", async () => {
  const code = 'function greet(name) {\n  return "Hello " + name;\n}';
  const result = await highlightCode(code, "javascript");
  assertEquals(stripAnsi(result), code);
});

Deno.test("empty input returns empty string", async () => {
  assertEquals(await highlightCode("", "typescript"), "");
  assertEquals(await highlightCode(""), "");
});

Deno.test("multiline structure preserved", async () => {
  const code = "if x > 0:\n    print(x)\n    return x";
  const result = await highlightCode(code, "python");
  const lines = result.split("\n");
  assertEquals(lines.length, 3);
  assertEquals(stripAnsi(result), code);
});
