import { assertEquals } from "@std/assert";
import { stripAnsi } from "./wrap.ts";
import {
  findMatches,
  highlightSearch,
  type Key,
  parseKey,
  truncateLine,
} from "./pager.ts";

// --- parseKey ---

Deno.test("parseKey: printable ASCII characters", () => {
  const keys = "abcqjkdunNG/ ";
  for (const ch of keys) {
    const key = parseKey(new Uint8Array([ch.charCodeAt(0)]));
    assertEquals(key, { type: "char", char: ch });
  }
});

Deno.test("parseKey: ctrl-c", () => {
  assertEquals(parseKey(new Uint8Array([0x03])), { type: "ctrl-c" });
});

Deno.test("parseKey: enter", () => {
  assertEquals(parseKey(new Uint8Array([0x0d])), { type: "enter" });
});

Deno.test("parseKey: backspace", () => {
  assertEquals(parseKey(new Uint8Array([0x7f])), { type: "backspace" });
});

Deno.test("parseKey: escape (bare)", () => {
  assertEquals(parseKey(new Uint8Array([0x1b])), { type: "escape" });
});

Deno.test("parseKey: arrow up", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x41])), { type: "up" });
});

Deno.test("parseKey: arrow down", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x42])), { type: "down" });
});

Deno.test("parseKey: home", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x48])), { type: "home" });
});

Deno.test("parseKey: end", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x46])), { type: "end" });
});

Deno.test("parseKey: page up", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x35, 0x7e])), {
    type: "pageup",
  });
});

Deno.test("parseKey: page down", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x36, 0x7e])), {
    type: "pagedown",
  });
});

Deno.test("parseKey: empty buffer", () => {
  assertEquals(parseKey(new Uint8Array([])), { type: "unknown" });
});

Deno.test("parseKey: unknown control char", () => {
  assertEquals(parseKey(new Uint8Array([0x01])), { type: "unknown" });
});

Deno.test("parseKey: incomplete CSI sequence", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b])), { type: "unknown" });
});

Deno.test("parseKey: unknown CSI sequence", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x5a])), {
    type: "unknown",
  });
});

// --- truncateLine ---

Deno.test("truncateLine: short line unchanged", () => {
  assertEquals(truncateLine("hello", 10), "hello");
});

Deno.test("truncateLine: exact width unchanged", () => {
  assertEquals(truncateLine("12345", 5), "12345");
});

Deno.test("truncateLine: long line truncated with ellipsis", () => {
  const result = truncateLine("hello world", 6);
  assertEquals(result, "hello…");
});

Deno.test("truncateLine: width 1 gives ellipsis", () => {
  assertEquals(truncateLine("hello", 1), "…");
});

Deno.test("truncateLine: preserves ANSI codes before truncation point", () => {
  const line = "\x1b[1mhello world\x1b[0m";
  const result = truncateLine(line, 6);
  // Should keep the bold code and truncate visible text
  assertEquals(result.includes("\x1b[1m"), true);
  assertEquals(stripAnsi(result), "hello…");
});

Deno.test("truncateLine: ANSI codes don't count toward width", () => {
  const line = "\x1b[31mhi\x1b[0m";
  const result = truncateLine(line, 10);
  assertEquals(result, line); // fits fine
});

// --- highlightSearch ---

Deno.test("highlightSearch: empty query returns line unchanged", () => {
  assertEquals(highlightSearch("hello world", ""), "hello world");
});

Deno.test("highlightSearch: no match returns line unchanged", () => {
  assertEquals(highlightSearch("hello world", "xyz"), "hello world");
});

Deno.test("highlightSearch: case-insensitive match", () => {
  const result = highlightSearch("Hello World", "hello");
  const plain = stripAnsi(result);
  assertEquals(plain, "Hello World");
  // Should contain reverse video codes
  assertEquals(result.includes("\x1b[7m"), true);
  assertEquals(result.includes("\x1b[27m"), true);
});

Deno.test("highlightSearch: highlights correct substring", () => {
  const result = highlightSearch("abcdef", "cd");
  assertEquals(result, "ab\x1b[7mcd\x1b[27mef");
});

Deno.test("highlightSearch: multiple matches highlighted", () => {
  const result = highlightSearch("abcabc", "ab");
  // Both "ab" occurrences should be highlighted
  const matches = result.match(/\x1b\[7m/g);
  assertEquals(matches?.length, 2);
});

Deno.test("highlightSearch: works with ANSI codes in line", () => {
  const line = "\x1b[1mhello\x1b[0m world";
  const result = highlightSearch(line, "hello");
  const plain = stripAnsi(result);
  assertEquals(plain, "hello world");
  // Should still have reverse video highlighting
  assertEquals(result.includes("\x1b[7m"), true);
});

Deno.test("highlightSearch: match at end of string", () => {
  const result = highlightSearch("foobar", "bar");
  assertEquals(result, "foo\x1b[7mbar\x1b[27m");
});

Deno.test("highlightSearch: match at start of string", () => {
  const result = highlightSearch("foobar", "foo");
  assertEquals(result, "\x1b[7mfoo\x1b[27mbar");
});

Deno.test("highlightSearch: entire string matches", () => {
  const result = highlightSearch("abc", "abc");
  assertEquals(result, "\x1b[7mabc\x1b[27m");
});

// --- findMatches ---

Deno.test("findMatches: empty query returns empty", () => {
  assertEquals(findMatches(["a", "b", "c"], ""), []);
});

Deno.test("findMatches: finds matching line indices", () => {
  const lines = ["hello world", "foo bar", "hello again"];
  assertEquals(findMatches(lines, "hello"), [0, 2]);
});

Deno.test("findMatches: case-insensitive", () => {
  const lines = ["Hello", "HELLO", "hello"];
  assertEquals(findMatches(lines, "hello"), [0, 1, 2]);
});

Deno.test("findMatches: no matches returns empty", () => {
  assertEquals(findMatches(["abc", "def"], "xyz"), []);
});

Deno.test("findMatches: ignores ANSI codes in lines", () => {
  const lines = ["\x1b[1mhello\x1b[0m", "world"];
  assertEquals(findMatches(lines, "hello"), [0]);
});

Deno.test("findMatches: single line match", () => {
  assertEquals(findMatches(["match"], "match"), [0]);
});
