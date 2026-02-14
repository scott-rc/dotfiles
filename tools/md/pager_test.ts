import { assertEquals } from "@std/assert";
import { stripAnsi, visibleLength } from "./wrap.ts";
import {
  findMatches,
  findNearestMatch,
  formatStatusBar,
  highlightSearch,
  type Key,
  mapScrollPosition,
  mapToSourceLine,
  parseKey,
  type StatusBarInput,
  truncateLine,
  wordBoundaryLeft,
  wordBoundaryRight,
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

Deno.test("parseKey: arrow left", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x44])), { type: "left" });
});

Deno.test("parseKey: arrow right", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x43])), { type: "right" });
});

Deno.test("parseKey: alt-left (CSI 1;3D)", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x31, 0x3b, 0x33, 0x44])), { type: "alt-left" });
});

Deno.test("parseKey: alt-right (CSI 1;3C)", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x5b, 0x31, 0x3b, 0x33, 0x43])), { type: "alt-right" });
});

Deno.test("parseKey: alt-left (ESC b)", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x62])), { type: "alt-left" });
});

Deno.test("parseKey: alt-right (ESC f)", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x66])), { type: "alt-right" });
});

Deno.test("parseKey: alt-backspace", () => {
  assertEquals(parseKey(new Uint8Array([0x1b, 0x7f])), { type: "alt-backspace" });
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

// --- mapToSourceLine ---

Deno.test("mapToSourceLine: top of file returns line 1", () => {
  const raw = "a\nb\nc\nd\ne";
  assertEquals(mapToSourceLine(0, 20, raw), 1);
});

Deno.test("mapToSourceLine: bottom of rendered maps to end of source", () => {
  const raw = "a\nb\nc\nd\ne"; // 5 source lines
  assertEquals(mapToSourceLine(20, 20, raw), 6);
});

Deno.test("mapToSourceLine: midpoint maps proportionally", () => {
  const raw = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10"; // 10 source lines
  const result = mapToSourceLine(50, 100, raw);
  assertEquals(result, 6); // 50% of 10 = 5, + 1 = 6
});

Deno.test("mapToSourceLine: rendered same length as source", () => {
  const raw = "a\nb\nc";
  assertEquals(mapToSourceLine(1, 3, raw), 2);
});

// --- mapScrollPosition ---

Deno.test("mapScrollPosition: preserves scroll ratio", () => {
  // At 50% through 101 lines → 50% through 201 lines
  assertEquals(mapScrollPosition(50, 101, 201), 100);
});

Deno.test("mapScrollPosition: top stays at top", () => {
  assertEquals(mapScrollPosition(0, 100, 200), 0);
});

Deno.test("mapScrollPosition: bottom maps to bottom", () => {
  assertEquals(mapScrollPosition(99, 100, 50), 49);
});

Deno.test("mapScrollPosition: single line stays at 0", () => {
  assertEquals(mapScrollPosition(0, 1, 50), 0);
});

Deno.test("mapScrollPosition: same line count preserves position", () => {
  assertEquals(mapScrollPosition(25, 100, 100), 25);
});

Deno.test("mapScrollPosition: shrink rounds to nearest", () => {
  // 3/9 = 0.333, * 4 = 1.333, rounds to 1
  assertEquals(mapScrollPosition(3, 10, 5), 1);
});

// --- findNearestMatch ---

Deno.test("findNearestMatch: finds first match at or after position", () => {
  assertEquals(findNearestMatch([5, 15, 25], 10), 1);
});

Deno.test("findNearestMatch: returns last if all before position", () => {
  assertEquals(findNearestMatch([5, 15, 25], 30), 2);
});

Deno.test("findNearestMatch: returns first if position is 0", () => {
  assertEquals(findNearestMatch([5, 15, 25], 0), 0);
});

Deno.test("findNearestMatch: empty matches returns -1", () => {
  assertEquals(findNearestMatch([], 10), -1);
});

Deno.test("findNearestMatch: exact position match", () => {
  assertEquals(findNearestMatch([5, 10, 15], 10), 1);
});

// --- formatStatusBar ---

function baseInput(overrides: Partial<StatusBarInput> = {}): StatusBarInput {
  return {
    mode: "normal",
    searchInput: "",
    searchCursor: 0,
    searchMessage: "",
    searchQuery: "",
    searchMatches: [],
    currentMatch: -1,
    topLine: 0,
    lineCount: 50,
    contentHeight: 24,
    filePath: "/path/to/README.md",
    ...overrides,
  };
}

Deno.test("formatStatusBar: normal mode with filename shows left/right layout", () => {
  const result = formatStatusBar(baseInput(), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.startsWith("README.md"), true);
  assertEquals(plain.includes("1-24/50"), true);
  assertEquals(plain.endsWith("TOP"), true);
});

Deno.test("formatStatusBar: normal mode without filename (stdin)", () => {
  const result = formatStatusBar(baseInput({ filePath: undefined }), 60);
  const plain = stripAnsi(result);
  // Left side should be empty, right side has position
  assertEquals(plain.includes("1-24/50"), true);
  assertEquals(plain.endsWith("TOP"), true);
});

Deno.test("formatStatusBar: at top shows TOP", () => {
  const result = formatStatusBar(baseInput({ topLine: 0 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.endsWith("TOP"), true);
});

Deno.test("formatStatusBar: at end shows END", () => {
  const result = formatStatusBar(baseInput({ topLine: 26 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.endsWith("END"), true);
});

Deno.test("formatStatusBar: short doc (top AND end) shows TOP", () => {
  const result = formatStatusBar(baseInput({ lineCount: 10 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.endsWith("TOP"), true);
});

Deno.test("formatStatusBar: scrolled mid-document shows percentage", () => {
  const result = formatStatusBar(baseInput({ topLine: 10 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.endsWith("68%"), true);
  assertEquals(plain.includes("11-34/50"), true);
});

Deno.test("formatStatusBar: search input mode shows cursor at end", () => {
  const result = formatStatusBar(baseInput({ mode: "search", searchInput: "query", searchCursor: 5 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.startsWith("/query\u2588"), true);
});

Deno.test("formatStatusBar: search cursor at beginning", () => {
  const result = formatStatusBar(baseInput({ mode: "search", searchInput: "query", searchCursor: 0 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.startsWith("/query"), true);
  // The cursor character should be 'q' (rendered in non-reverse)
  assertEquals(result.includes("\x1b[27mq\x1b[7m"), true);
});

Deno.test("formatStatusBar: search cursor in middle", () => {
  const result = formatStatusBar(baseInput({ mode: "search", searchInput: "query", searchCursor: 2 }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.startsWith("/query"), true);
  // The cursor character should be 'e' (at index 2)
  assertEquals(result.includes("\x1b[27me\x1b[7m"), true);
});

Deno.test("formatStatusBar: search message shows message only", () => {
  const result = formatStatusBar(baseInput({ searchMessage: "Copied: README.md" }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.startsWith("Copied: README.md"), true);
  // No position info on right
  assertEquals(plain.includes("/50"), false);
});

Deno.test("formatStatusBar: active search with results", () => {
  const result = formatStatusBar(baseInput({
    searchQuery: "hello",
    searchMatches: [5, 15, 25, 35, 45],
    currentMatch: 1,
  }), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.startsWith("/hello (2/5)"), true);
  assertEquals(plain.includes("1-24/50"), true);
});

Deno.test("formatStatusBar: narrow terminal preserves right side", () => {
  const result = formatStatusBar(baseInput(), 30);
  const plain = stripAnsi(result);
  // Right side (position) should still be present
  assertEquals(plain.includes("TOP"), true);
  assertEquals(plain.includes("1-24/50"), true);
});

Deno.test("formatStatusBar: very narrow terminal graceful degradation", () => {
  const result = formatStatusBar(baseInput(), 10);
  // Should not throw; output should have some content
  const plain = stripAnsi(result);
  assertEquals(plain.length > 0, true);
});

Deno.test("formatStatusBar: line range info is dimmed", () => {
  const result = formatStatusBar(baseInput({ topLine: 10 }), 60);
  // DIM (SGR 2) should appear before line range
  assertEquals(result.includes("\x1b[2m"), true);
  // NO_DIM (SGR 22) should appear after line range
  assertEquals(result.includes("\x1b[22m"), true);
});

Deno.test("formatStatusBar: visible width matches cols exactly", () => {
  for (const cols of [40, 60, 80, 120]) {
    const result = formatStatusBar(baseInput({ topLine: 10 }), cols);
    assertEquals(visibleLength(result), cols);
  }
});

// --- wordBoundaryLeft ---

Deno.test("wordBoundaryLeft: end to start of last word", () => {
  assertEquals(wordBoundaryLeft("hello world", 11), 6);
});

Deno.test("wordBoundaryLeft: start of word to start of previous word", () => {
  assertEquals(wordBoundaryLeft("hello world", 6), 0);
});

Deno.test("wordBoundaryLeft: mid-word to start of that word", () => {
  assertEquals(wordBoundaryLeft("hello world", 3), 0);
});

Deno.test("wordBoundaryLeft: consecutive spaces", () => {
  assertEquals(wordBoundaryLeft("hello  world", 12), 7);
});

Deno.test("wordBoundaryLeft: already at start", () => {
  assertEquals(wordBoundaryLeft("hello", 0), 0);
});

// --- wordBoundaryRight ---

Deno.test("wordBoundaryRight: start to start of next word", () => {
  assertEquals(wordBoundaryRight("hello world", 0), 6);
});

Deno.test("wordBoundaryRight: start of last word to end", () => {
  assertEquals(wordBoundaryRight("hello world", 6), 11);
});

Deno.test("wordBoundaryRight: consecutive spaces", () => {
  assertEquals(wordBoundaryRight("hello  world", 0), 7);
});

Deno.test("wordBoundaryRight: already at end", () => {
  assertEquals(wordBoundaryRight("hello", 5), 5);
});
