import { assertEquals } from "@std/assert";
import { stripAnsi, visibleLength, wordWrap } from "./wrap.ts";

Deno.test("stripAnsi removes ANSI codes", () => {
  assertEquals(stripAnsi("\x1b[1mhello\x1b[0m"), "hello");
  assertEquals(stripAnsi("\x1b[31;1mbold red\x1b[0m"), "bold red");
  assertEquals(stripAnsi("no codes"), "no codes");
});

Deno.test("visibleLength ignores ANSI codes", () => {
  assertEquals(visibleLength("\x1b[1mhello\x1b[0m"), 5);
  assertEquals(visibleLength("hello"), 5);
  assertEquals(visibleLength(""), 0);
});

Deno.test("wordWrap wraps at word boundaries", () => {
  const result = wordWrap("the quick brown fox jumps over the lazy dog", 20);
  // Widow prevention pulls "lazy" down to join "dog" on the last line
  assertEquals(result, "the quick brown\nfox jumps over the\nlazy dog");
});

Deno.test("wordWrap respects indent prefix", () => {
  const result = wordWrap("hello world foo", 15, "  ");
  // Widow prevention pulls "world" down to join "foo"
  assertEquals(result, "  hello\n  world foo");
});

Deno.test("wordWrap handles ANSI codes without counting toward width", () => {
  const bold = "\x1b[1m";
  const reset = "\x1b[0m";
  const text = `${bold}hello${reset} world`;
  const result = wordWrap(text, 11);
  assertEquals(result, `${bold}hello${reset} world`);
});

Deno.test("wordWrap preserves existing newlines", () => {
  const result = wordWrap("line one\nline two", 80);
  assertEquals(result, "line one\nline two");
});

Deno.test("wordWrap handles empty string", () => {
  assertEquals(wordWrap("", 80), "");
});

Deno.test("wordWrap handles single word longer than width", () => {
  const result = wordWrap("superlongword", 5);
  assertEquals(result, "super\nlongw\nord");
});

Deno.test("wordWrap handles text exactly at width", () => {
  const result = wordWrap("12345", 5);
  assertEquals(result, "12345");
});

Deno.test("wordWrap avoids widow (single word on last line)", () => {
  const result = wordWrap("the quick brown fox jumps over the lazy dog", 20);
  const lines = result.split("\n");
  const lastWords = lines[lines.length - 1].trim().split(/\s+/);
  assertEquals(lastWords.length >= 2, true);
});

Deno.test("wordWrap keeps opening backtick with code content", () => {
  // Simulate styled code span: gray(`) + orange(code) + gray(`)
  const gray = "\x1b[38;2;139;148;158m";
  const orange = "\x1b[38;2;255;166;87m";
  const reset = "\x1b[39m";
  const codeSpan = `${gray}\`${reset}${orange}code${reset}${gray}\`${reset}`;
  const text = `some text ${codeSpan} end`;
  // "some text `" = 11 chars, "code" = 4 more → 15 > 12, wraps.
  // Without fix: line 1 ends with dangling "`"
  // With fix: "`code`" stays together on line 2
  const result = wordWrap(text, 12);
  const lines = result.split("\n");
  const firstVisible = stripAnsi(lines[0]).trimEnd();
  assertEquals(firstVisible.endsWith("`"), false);
  assertEquals(firstVisible, "some text");
});
