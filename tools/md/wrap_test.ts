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
  assertEquals(result, "the quick brown fox\njumps over the lazy\ndog");
});

Deno.test("wordWrap respects indent prefix", () => {
  const result = wordWrap("hello world foo", 15, "  ");
  assertEquals(result, "  hello world\n  foo");
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
