/**
 * Generate language-agnostic test fixtures from the current implementation.
 *
 * Run: deno task generate-fixtures
 *
 * Produces:
 *   fixtures/rendering/*.md + *.expected.txt  (ANSI-stripped, width 60)
 *   fixtures/wrapping/*.json
 *   fixtures/pager/*.json
 *   fixtures/browse/*.json
 */

import { renderMarkdown } from "./mod.ts";
import { stripAnsi, visibleLength, wordWrap } from "./wrap.ts";
import {
  findMatches,
  findNearestMatch,
  formatStatusBar,
  handleSearchKey,
  highlightSearch,
  mapScrollPosition,
  mapToSourceLine,
  parseKey,
  type PagerState,
  type StatusBarInput,
  truncateLine,
  wordBoundaryLeft,
  wordBoundaryRight,
} from "./pager.ts";
import {
  buildFindCmd,
  buildPickCmd,
  parseSelection,
  shellQuote,
  shouldPage,
} from "./browse.ts";

// ── Helpers ──────────────────────────────────────────────

const FIXTURE_DIR = new URL("./fixtures/", import.meta.url);

async function writeText(path: string, content: string) {
  const url = new URL(path, FIXTURE_DIR);
  await Deno.mkdir(new URL(".", url), { recursive: true });
  await Deno.writeTextFile(url, content);
}

async function writeJSON(path: string, data: unknown) {
  await writeText(path, JSON.stringify(data, null, 2) + "\n");
}

// ── Rendering Fixtures ───────────────────────────────────

const RENDERING_INPUTS: Record<string, string> = {
  "heading-h1": "# Hello World",
  "heading-h2": "## Section Title",
  "bold": "**bold text**",
  "italic": "*italic text*",
  "inline-code": "use `foo` here",
  "code-block-plain": "```\nhello\n```",
  "code-block-lang": "```typescript\nconst x = 1;\n```",
  "unordered-list": "- one\n- two\n- three",
  "ordered-list": "1. first\n2. second\n3. third",
  "nested-list": "- outer\n  - inner\n  - inner2\n- outer2",
  "blockquote": "> quoted text here",
  "link": "[example](https://example.com)",
  "hr": "---",
  "paragraph-wrap":
    "The quick brown fox jumps over the lazy dog and continues running through the forest until it finds a peaceful meadow to rest.",
  "mixed-document": `# Title

Some **bold** and *italic* text.

## Code Example

\`\`\`js
const x = 1;
\`\`\`

- item one
- item two

> a quote

---

[link](https://example.com)`,
  "frontmatter-basic": `---
title: My Doc
date: 2024-01-01
---

# Hello`,
  "frontmatter-arrays": `---
tags:
  - one
  - two
  - three
---

# Tags`,
  "frontmatter-empty": `---
---

# Hello`,
  "frontmatter-malformed": `---
: [[[invalid
---

# Hello`,
  "bare-hr-not-frontmatter": `---

Some text.`,
};

async function generateRenderingFixtures() {
  for (const [name, input] of Object.entries(RENDERING_INPUTS)) {
    await writeText(`rendering/${name}.md`, input);
    const rendered = await renderMarkdown(input, { width: 60 });
    await writeText(`rendering/${name}.expected.txt`, stripAnsi(rendered));
  }
}

// ── Wrapping Fixtures ────────────────────────────────────

function generateWrappingFixtures() {
  const cases = [
    {
      name: "basic wrap at word boundaries",
      input: "the quick brown fox jumps over the lazy dog",
      params: { width: 20 },
      expected: wordWrap("the quick brown fox jumps over the lazy dog", 20),
    },
    {
      name: "respects indent prefix",
      input: "hello world foo",
      params: { width: 15, indent: "  " },
      expected: wordWrap("hello world foo", 15, "  "),
    },
    {
      name: "ANSI codes not counted toward width",
      input: "\x1b[1mhello\x1b[0m world",
      params: { width: 11 },
      expected: wordWrap("\x1b[1mhello\x1b[0m world", 11),
    },
    {
      name: "preserves existing newlines",
      input: "line one\nline two",
      params: { width: 80 },
      expected: wordWrap("line one\nline two", 80),
    },
    {
      name: "empty string",
      input: "",
      params: { width: 80 },
      expected: wordWrap("", 80),
    },
    {
      name: "single word longer than width",
      input: "superlongword",
      params: { width: 5 },
      expected: wordWrap("superlongword", 5),
    },
    {
      name: "text exactly at width",
      input: "12345",
      params: { width: 5 },
      expected: wordWrap("12345", 5),
    },
    {
      name: "widow prevention",
      input: "the quick brown fox jumps over the lazy dog",
      params: { width: 20 },
      expected: wordWrap("the quick brown fox jumps over the lazy dog", 20),
    },
    {
      name: "backtick-aware line breaking",
      input: (() => {
        const gray = "\x1b[38;2;139;148;158m";
        const orange = "\x1b[38;2;255;166;87m";
        const reset = "\x1b[39m";
        return `some text ${gray}\`${reset}${orange}code${reset}${gray}\`${reset} end`;
      })(),
      params: { width: 12 },
      expected: (() => {
        const gray = "\x1b[38;2;139;148;158m";
        const orange = "\x1b[38;2;255;166;87m";
        const reset = "\x1b[39m";
        return wordWrap(
          `some text ${gray}\`${reset}${orange}code${reset}${gray}\`${reset} end`,
          12,
        );
      })(),
    },
  ];

  const stripAnsiCases = [
    { name: "removes bold", input: "\x1b[1mhello\x1b[0m", expected: "hello" },
    {
      name: "removes compound codes",
      input: "\x1b[31;1mbold red\x1b[0m",
      expected: "bold red",
    },
    { name: "no-op on plain text", input: "no codes", expected: "no codes" },
  ];

  const visibleLengthCases = [
    { name: "with ANSI codes", input: "\x1b[1mhello\x1b[0m", expected: 5 },
    { name: "plain text", input: "hello", expected: 5 },
    { name: "empty string", input: "", expected: 0 },
  ];

  return writeJSON("wrapping/word-wrap.json", {
    wordWrap: cases,
    stripAnsi: stripAnsiCases,
    visibleLength: visibleLengthCases,
  });
}

// ── Pager Fixtures ───────────────────────────────────────

function generateParseKeyFixtures() {
  const cases = [
    // Printable ASCII
    ...["a", "b", "c", "q", "j", "k", "d", "u", "n", "N", "G", "/", " "].map(
      (ch) => ({
        name: `printable '${ch}'`,
        input: [ch.charCodeAt(0)],
        expected: parseKey(new Uint8Array([ch.charCodeAt(0)])),
      }),
    ),
    // Control characters
    { name: "ctrl-c", input: [0x03], expected: { type: "ctrl-c" } },
    { name: "ctrl-d", input: [0x04], expected: { type: "ctrl-d" } },
    { name: "enter", input: [0x0d], expected: { type: "enter" } },
    { name: "ctrl-u", input: [0x15], expected: { type: "ctrl-u" } },
    { name: "backspace", input: [0x7f], expected: { type: "backspace" } },
    // Escape
    { name: "escape (bare)", input: [0x1b], expected: { type: "escape" } },
    // Arrow keys
    {
      name: "arrow up",
      input: [0x1b, 0x5b, 0x41],
      expected: { type: "up" },
    },
    {
      name: "arrow down",
      input: [0x1b, 0x5b, 0x42],
      expected: { type: "down" },
    },
    {
      name: "arrow right",
      input: [0x1b, 0x5b, 0x43],
      expected: { type: "right" },
    },
    {
      name: "arrow left",
      input: [0x1b, 0x5b, 0x44],
      expected: { type: "left" },
    },
    // Home/End
    { name: "home", input: [0x1b, 0x5b, 0x48], expected: { type: "home" } },
    { name: "end", input: [0x1b, 0x5b, 0x46], expected: { type: "end" } },
    // Page Up/Down
    {
      name: "page up",
      input: [0x1b, 0x5b, 0x35, 0x7e],
      expected: { type: "pageup" },
    },
    {
      name: "page down",
      input: [0x1b, 0x5b, 0x36, 0x7e],
      expected: { type: "pagedown" },
    },
    // Alt keys
    {
      name: "alt-left (CSI 1;3D)",
      input: [0x1b, 0x5b, 0x31, 0x3b, 0x33, 0x44],
      expected: { type: "alt-left" },
    },
    {
      name: "alt-right (CSI 1;3C)",
      input: [0x1b, 0x5b, 0x31, 0x3b, 0x33, 0x43],
      expected: { type: "alt-right" },
    },
    {
      name: "alt-left (ESC b)",
      input: [0x1b, 0x62],
      expected: { type: "alt-left" },
    },
    {
      name: "alt-right (ESC f)",
      input: [0x1b, 0x66],
      expected: { type: "alt-right" },
    },
    {
      name: "alt-backspace",
      input: [0x1b, 0x7f],
      expected: { type: "alt-backspace" },
    },
    // Edge cases
    { name: "empty buffer", input: [], expected: { type: "unknown" } },
    {
      name: "unknown control char",
      input: [0x01],
      expected: { type: "unknown" },
    },
    {
      name: "incomplete CSI sequence",
      input: [0x1b, 0x5b],
      expected: { type: "unknown" },
    },
    {
      name: "unknown CSI sequence",
      input: [0x1b, 0x5b, 0x5a],
      expected: { type: "unknown" },
    },
  ];
  return writeJSON("pager/parse-key.json", cases);
}

function generateTruncateLineFixtures() {
  const cases = [
    {
      name: "short line unchanged",
      input: "hello",
      params: { maxWidth: 10 },
      expected: "hello",
    },
    {
      name: "exact width unchanged",
      input: "12345",
      params: { maxWidth: 5 },
      expected: "12345",
    },
    {
      name: "long line truncated with ellipsis",
      input: "hello world",
      params: { maxWidth: 6 },
      expected: truncateLine("hello world", 6),
    },
    {
      name: "width 1 gives ellipsis",
      input: "hello",
      params: { maxWidth: 1 },
      expected: "…",
    },
    {
      name: "preserves ANSI codes before truncation point",
      input: "\x1b[1mhello world\x1b[0m",
      params: { maxWidth: 6 },
      expected: truncateLine("\x1b[1mhello world\x1b[0m", 6),
    },
    {
      name: "ANSI codes don't count toward width",
      input: "\x1b[31mhi\x1b[0m",
      params: { maxWidth: 10 },
      expected: "\x1b[31mhi\x1b[0m",
    },
  ];
  return writeJSON("pager/truncate-line.json", cases);
}

function generateHighlightSearchFixtures() {
  const cases = [
    {
      name: "empty query returns line unchanged",
      input: "hello world",
      params: { query: "" },
      expected: "hello world",
    },
    {
      name: "no match returns line unchanged",
      input: "hello world",
      params: { query: "xyz" },
      expected: "hello world",
    },
    {
      name: "case-insensitive match",
      input: "Hello World",
      params: { query: "hello" },
      expected: highlightSearch("Hello World", "hello"),
    },
    {
      name: "highlights correct substring",
      input: "abcdef",
      params: { query: "cd" },
      expected: highlightSearch("abcdef", "cd"),
    },
    {
      name: "multiple matches highlighted",
      input: "abcabc",
      params: { query: "ab" },
      expected: highlightSearch("abcabc", "ab"),
    },
    {
      name: "works with ANSI codes in line",
      input: "\x1b[1mhello\x1b[0m world",
      params: { query: "hello" },
      expected: highlightSearch("\x1b[1mhello\x1b[0m world", "hello"),
    },
    {
      name: "match at end of string",
      input: "foobar",
      params: { query: "bar" },
      expected: highlightSearch("foobar", "bar"),
    },
    {
      name: "match at start of string",
      input: "foobar",
      params: { query: "foo" },
      expected: highlightSearch("foobar", "foo"),
    },
    {
      name: "entire string matches",
      input: "abc",
      params: { query: "abc" },
      expected: highlightSearch("abc", "abc"),
    },
  ];
  return writeJSON("pager/highlight-search.json", cases);
}

function generateFindMatchesFixtures() {
  const cases = [
    {
      name: "empty query returns empty",
      input: { lines: ["a", "b", "c"], query: "" },
      expected: [] as number[],
    },
    {
      name: "finds matching line indices",
      input: {
        lines: ["hello world", "foo bar", "hello again"],
        query: "hello",
      },
      expected: [0, 2],
    },
    {
      name: "case-insensitive",
      input: { lines: ["Hello", "HELLO", "hello"], query: "hello" },
      expected: [0, 1, 2],
    },
    {
      name: "no matches returns empty",
      input: { lines: ["abc", "def"], query: "xyz" },
      expected: [] as number[],
    },
    {
      name: "ignores ANSI codes in lines",
      input: { lines: ["\x1b[1mhello\x1b[0m", "world"], query: "hello" },
      expected: [0],
    },
    {
      name: "single line match",
      input: { lines: ["match"], query: "match" },
      expected: [0],
    },
  ];
  return writeJSON("pager/find-matches.json", cases);
}

function generateMapScrollPositionFixtures() {
  const cases = [
    {
      name: "preserves scroll ratio",
      input: { oldTopLine: 50, oldLineCount: 101, newLineCount: 201 },
      expected: 100,
    },
    {
      name: "top stays at top",
      input: { oldTopLine: 0, oldLineCount: 100, newLineCount: 200 },
      expected: 0,
    },
    {
      name: "bottom maps to bottom",
      input: { oldTopLine: 99, oldLineCount: 100, newLineCount: 50 },
      expected: 49,
    },
    {
      name: "single line stays at 0",
      input: { oldTopLine: 0, oldLineCount: 1, newLineCount: 50 },
      expected: 0,
    },
    {
      name: "same line count preserves position",
      input: { oldTopLine: 25, oldLineCount: 100, newLineCount: 100 },
      expected: 25,
    },
    {
      name: "shrink rounds to nearest",
      input: { oldTopLine: 3, oldLineCount: 10, newLineCount: 5 },
      expected: 1,
    },
  ];
  return writeJSON("pager/map-scroll-position.json", cases);
}

function generateFindNearestMatchFixtures() {
  const cases = [
    {
      name: "finds first match at or after position",
      input: { matches: [5, 15, 25], topLine: 10 },
      expected: 1,
    },
    {
      name: "returns last if all before position",
      input: { matches: [5, 15, 25], topLine: 30 },
      expected: 2,
    },
    {
      name: "returns first if position is 0",
      input: { matches: [5, 15, 25], topLine: 0 },
      expected: 0,
    },
    {
      name: "empty matches returns -1",
      input: { matches: [] as number[], topLine: 10 },
      expected: -1,
    },
    {
      name: "exact position match",
      input: { matches: [5, 10, 15], topLine: 10 },
      expected: 1,
    },
  ];
  return writeJSON("pager/find-nearest-match.json", cases);
}

function baseStatusInput(
  overrides: Partial<StatusBarInput> = {},
): StatusBarInput {
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

function generateFormatStatusBarFixtures() {
  const cases = [
    {
      name: "normal mode with filename",
      input: { state: baseStatusInput(), cols: 60 },
      expected: formatStatusBar(baseStatusInput(), 60),
    },
    {
      name: "normal mode without filename (stdin)",
      input: {
        state: baseStatusInput({ filePath: undefined }),
        cols: 60,
      },
      expected: formatStatusBar(baseStatusInput({ filePath: undefined }), 60),
    },
    {
      name: "at top shows TOP",
      input: { state: baseStatusInput({ topLine: 0 }), cols: 60 },
      expected: formatStatusBar(baseStatusInput({ topLine: 0 }), 60),
    },
    {
      name: "at end shows END",
      input: { state: baseStatusInput({ topLine: 26 }), cols: 60 },
      expected: formatStatusBar(baseStatusInput({ topLine: 26 }), 60),
    },
    {
      name: "short doc shows TOP",
      input: { state: baseStatusInput({ lineCount: 10 }), cols: 60 },
      expected: formatStatusBar(baseStatusInput({ lineCount: 10 }), 60),
    },
    {
      name: "scrolled mid-document shows percentage",
      input: { state: baseStatusInput({ topLine: 10 }), cols: 60 },
      expected: formatStatusBar(baseStatusInput({ topLine: 10 }), 60),
    },
    {
      name: "search input mode",
      input: {
        state: baseStatusInput({
          mode: "search",
          searchInput: "query",
          searchCursor: 5,
        }),
        cols: 60,
      },
      expected: formatStatusBar(
        baseStatusInput({
          mode: "search",
          searchInput: "query",
          searchCursor: 5,
        }),
        60,
      ),
    },
    {
      name: "search cursor at beginning",
      input: {
        state: baseStatusInput({
          mode: "search",
          searchInput: "query",
          searchCursor: 0,
        }),
        cols: 60,
      },
      expected: formatStatusBar(
        baseStatusInput({
          mode: "search",
          searchInput: "query",
          searchCursor: 0,
        }),
        60,
      ),
    },
    {
      name: "search message",
      input: {
        state: baseStatusInput({ searchMessage: "Copied: README.md" }),
        cols: 60,
      },
      expected: formatStatusBar(
        baseStatusInput({ searchMessage: "Copied: README.md" }),
        60,
      ),
    },
    {
      name: "active search with results",
      input: {
        state: baseStatusInput({
          searchQuery: "hello",
          searchMatches: [5, 15, 25, 35, 45],
          currentMatch: 1,
        }),
        cols: 60,
      },
      expected: formatStatusBar(
        baseStatusInput({
          searchQuery: "hello",
          searchMatches: [5, 15, 25, 35, 45],
          currentMatch: 1,
        }),
        60,
      ),
    },
    {
      name: "narrow terminal",
      input: { state: baseStatusInput(), cols: 30 },
      expected: formatStatusBar(baseStatusInput(), 30),
    },
    {
      name: "very narrow terminal",
      input: { state: baseStatusInput(), cols: 10 },
      expected: formatStatusBar(baseStatusInput(), 10),
    },
    {
      name: "width 40",
      input: { state: baseStatusInput({ topLine: 10 }), cols: 40 },
      expected: formatStatusBar(baseStatusInput({ topLine: 10 }), 40),
    },
    {
      name: "width 80",
      input: { state: baseStatusInput({ topLine: 10 }), cols: 80 },
      expected: formatStatusBar(baseStatusInput({ topLine: 10 }), 80),
    },
    {
      name: "width 120",
      input: { state: baseStatusInput({ topLine: 10 }), cols: 120 },
      expected: formatStatusBar(baseStatusInput({ topLine: 10 }), 120),
    },
  ];
  return writeJSON("pager/format-status-bar.json", cases);
}

function generateWordBoundaryFixtures() {
  const cases = {
    left: [
      {
        name: "end to start of last word",
        input: { text: "hello world", cursor: 11 },
        expected: 6,
      },
      {
        name: "start of word to start of previous",
        input: { text: "hello world", cursor: 6 },
        expected: 0,
      },
      {
        name: "mid-word to start of that word",
        input: { text: "hello world", cursor: 3 },
        expected: 0,
      },
      {
        name: "consecutive spaces",
        input: { text: "hello  world", cursor: 12 },
        expected: 7,
      },
      {
        name: "already at start",
        input: { text: "hello", cursor: 0 },
        expected: 0,
      },
    ],
    right: [
      {
        name: "start to start of next word",
        input: { text: "hello world", cursor: 0 },
        expected: 6,
      },
      {
        name: "start of last word to end",
        input: { text: "hello world", cursor: 6 },
        expected: 11,
      },
      {
        name: "consecutive spaces",
        input: { text: "hello  world", cursor: 0 },
        expected: 7,
      },
      {
        name: "already at end",
        input: { text: "hello", cursor: 5 },
        expected: 5,
      },
    ],
  };
  return writeJSON("pager/word-boundary.json", cases);
}

function makeSearchState(overrides: Partial<PagerState> = {}): PagerState {
  return {
    lines: ["hello world", "foo bar", "baz"],
    topLine: 0,
    searchQuery: "",
    searchMatches: [],
    currentMatch: -1,
    mode: "search",
    searchInput: "",
    searchCursor: 0,
    searchMessage: "",
    ...overrides,
  };
}

function generateHandleSearchKeyFixtures() {
  // For each case: define initial state, key, and expected state changes
  const cases = [
    (() => {
      const state = makeSearchState();
      handleSearchKey(state, { type: "backspace" });
      return {
        name: "backspace on empty input exits search",
        state: makeSearchState(),
        key: { type: "backspace" },
        expected: { mode: state.mode, searchInput: state.searchInput },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "a",
        searchCursor: 1,
      });
      handleSearchKey(state, { type: "backspace" });
      return {
        name: "backspace to empty exits search",
        state: makeSearchState({ searchInput: "a", searchCursor: 1 }),
        key: { type: "backspace" },
        expected: {
          mode: state.mode,
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "ab",
        searchCursor: 2,
      });
      handleSearchKey(state, { type: "backspace" });
      return {
        name: "backspace mid-input stays in search",
        state: makeSearchState({ searchInput: "ab", searchCursor: 2 }),
        key: { type: "backspace" },
        expected: {
          mode: state.mode,
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "hello",
        searchCursor: 5,
      });
      handleSearchKey(state, { type: "alt-backspace" });
      return {
        name: "alt-backspace to empty exits search",
        state: makeSearchState({ searchInput: "hello", searchCursor: 5 }),
        key: { type: "alt-backspace" },
        expected: { mode: state.mode, searchInput: state.searchInput },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "hello",
        searchCursor: 5,
      });
      handleSearchKey(state, { type: "ctrl-u" });
      return {
        name: "ctrl-u to empty exits search",
        state: makeSearchState({ searchInput: "hello", searchCursor: 5 }),
        key: { type: "ctrl-u" },
        expected: { mode: state.mode, searchInput: state.searchInput },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "hello",
        searchCursor: 0,
      });
      handleSearchKey(state, { type: "ctrl-u" });
      return {
        name: "ctrl-u with text after cursor stays in search",
        state: makeSearchState({ searchInput: "hello", searchCursor: 0 }),
        key: { type: "ctrl-u" },
        expected: {
          mode: state.mode,
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "test",
        searchCursor: 4,
      });
      handleSearchKey(state, { type: "char", char: "x" });
      return {
        name: "char appends to input",
        state: makeSearchState({ searchInput: "test", searchCursor: 4 }),
        key: { type: "char", char: "x" },
        expected: {
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "test",
        searchCursor: 2,
      });
      handleSearchKey(state, { type: "char", char: "x" });
      return {
        name: "char inserts at cursor",
        state: makeSearchState({ searchInput: "test", searchCursor: 2 }),
        key: { type: "char", char: "x" },
        expected: {
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "test",
        searchCursor: 2,
      });
      handleSearchKey(state, { type: "left" });
      return {
        name: "left moves cursor",
        state: makeSearchState({ searchInput: "test", searchCursor: 2 }),
        key: { type: "left" },
        expected: { searchCursor: state.searchCursor },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "test",
        searchCursor: 2,
      });
      handleSearchKey(state, { type: "right" });
      return {
        name: "right moves cursor",
        state: makeSearchState({ searchInput: "test", searchCursor: 2 }),
        key: { type: "right" },
        expected: { searchCursor: state.searchCursor },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "hello world",
        searchCursor: 11,
      });
      handleSearchKey(state, { type: "alt-left" });
      return {
        name: "alt-left moves to word boundary",
        state: makeSearchState({
          searchInput: "hello world",
          searchCursor: 11,
        }),
        key: { type: "alt-left" },
        expected: { searchCursor: state.searchCursor },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "hello world",
        searchCursor: 0,
      });
      handleSearchKey(state, { type: "alt-right" });
      return {
        name: "alt-right moves to word boundary",
        state: makeSearchState({
          searchInput: "hello world",
          searchCursor: 0,
        }),
        key: { type: "alt-right" },
        expected: { searchCursor: state.searchCursor },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "test",
        searchCursor: 4,
      });
      handleSearchKey(state, { type: "escape" });
      return {
        name: "escape cancels search",
        state: makeSearchState({ searchInput: "test", searchCursor: 4 }),
        key: { type: "escape" },
        expected: {
          mode: state.mode,
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
    (() => {
      const state = makeSearchState({
        searchInput: "test",
        searchCursor: 4,
      });
      handleSearchKey(state, { type: "ctrl-c" });
      return {
        name: "ctrl-c cancels search",
        state: makeSearchState({ searchInput: "test", searchCursor: 4 }),
        key: { type: "ctrl-c" },
        expected: {
          mode: state.mode,
          searchInput: state.searchInput,
          searchCursor: state.searchCursor,
        },
      };
    })(),
  ];
  return writeJSON("pager/handle-search-key.json", cases);
}

function generateMapToSourceLineFixtures() {
  const cases = [
    {
      name: "top of file returns line 1",
      input: { topLine: 0, renderedLineCount: 20, rawContent: "a\nb\nc\nd\ne" },
      expected: 1,
    },
    {
      name: "bottom maps to end of source",
      input: {
        topLine: 20,
        renderedLineCount: 20,
        rawContent: "a\nb\nc\nd\ne",
      },
      expected: mapToSourceLine(20, 20, "a\nb\nc\nd\ne"),
    },
    {
      name: "midpoint maps proportionally",
      input: {
        topLine: 50,
        renderedLineCount: 100,
        rawContent: "1\n2\n3\n4\n5\n6\n7\n8\n9\n10",
      },
      expected: mapToSourceLine(50, 100, "1\n2\n3\n4\n5\n6\n7\n8\n9\n10"),
    },
    {
      name: "rendered same length as source",
      input: { topLine: 1, renderedLineCount: 3, rawContent: "a\nb\nc" },
      expected: mapToSourceLine(1, 3, "a\nb\nc"),
    },
  ];
  return writeJSON("pager/map-to-source-line.json", cases);
}

// ── Browse Fixtures ──────────────────────────────────────

function generateBrowseFixtures() {
  const shellQuoteCases = [
    { name: "simple string", input: "hello", expected: "'hello'" },
    {
      name: "string with spaces",
      input: "hello world",
      expected: "'hello world'",
    },
    {
      name: "string with single quotes",
      input: "it's",
      expected: "'it'\\''s'",
    },
    { name: "empty string", input: "", expected: "''" },
    {
      name: "multiple single quotes",
      input: "a'b'c",
      expected: "'a'\\''b'\\''c'",
    },
  ];

  const buildFindCmdCases = [
    {
      name: "default template",
      input: { dir: "/some/dir" },
      expected: buildFindCmd("/some/dir"),
    },
    {
      name: "custom template with {dir}",
      input: { dir: "/docs", template: "find {dir} -name '*.md'" },
      expected: buildFindCmd("/docs", "find {dir} -name '*.md'"),
    },
    {
      name: "dir with spaces",
      input: { dir: "/my docs/notes" },
      expected: buildFindCmd("/my docs/notes"),
    },
    {
      name: "appends dir when no {dir} placeholder",
      input: { dir: "/docs", template: "fd -e md" },
      expected: buildFindCmd("/docs", "fd -e md"),
    },
  ];

  const buildPickCmdCases = [
    { name: "default", input: {}, expected: "fzf" },
    {
      name: "custom",
      input: { template: "fzf --exact" },
      expected: "fzf --exact",
    },
  ];

  const parseSelectionCases = [
    {
      name: "trims and returns path",
      input: "  docs/README.md  \n",
      expected: "docs/README.md",
    },
    { name: "empty string returns null", input: "", expected: null },
    {
      name: "whitespace-only returns null",
      input: "   \n  ",
      expected: null,
    },
    {
      name: "normal path as-is",
      input: "src/main.ts",
      expected: "src/main.ts",
    },
  ];

  const shouldPageCases = [
    {
      name: "returns false when --no-pager",
      input: {
        noPager: true,
        isTTY: true,
        contentLines: 100,
        terminalRows: 24,
        browsing: false,
      },
      expected: false,
    },
    {
      name: "returns false when not a TTY",
      input: {
        noPager: false,
        isTTY: false,
        contentLines: 100,
        terminalRows: 24,
        browsing: false,
      },
      expected: false,
    },
    {
      name: "returns true when content exceeds terminal",
      input: {
        noPager: false,
        isTTY: true,
        contentLines: 100,
        terminalRows: 24,
        browsing: false,
      },
      expected: true,
    },
    {
      name: "returns false when content fits",
      input: {
        noPager: false,
        isTTY: true,
        contentLines: 10,
        terminalRows: 24,
        browsing: false,
      },
      expected: false,
    },
    {
      name: "returns true when browsing even if content fits",
      input: {
        noPager: false,
        isTTY: true,
        contentLines: 5,
        terminalRows: 24,
        browsing: true,
      },
      expected: true,
    },
    {
      name: "respects --no-pager even when browsing",
      input: {
        noPager: true,
        isTTY: true,
        contentLines: 5,
        terminalRows: 24,
        browsing: true,
      },
      expected: false,
    },
  ];

  return Promise.all([
    writeJSON("browse/shell-quote.json", shellQuoteCases),
    writeJSON("browse/build-find-cmd.json", buildFindCmdCases),
    writeJSON("browse/build-pick-cmd.json", buildPickCmdCases),
    writeJSON("browse/parse-selection.json", parseSelectionCases),
    writeJSON("browse/should-page.json", shouldPageCases),
  ]);
}

// ── Main ─────────────────────────────────────────────────

if (import.meta.main) {
  console.log("Generating fixtures...");

  await generateRenderingFixtures();
  console.log("  ✓ rendering");

  await generateWrappingFixtures();
  console.log("  ✓ wrapping");

  await Promise.all([
    generateParseKeyFixtures(),
    generateTruncateLineFixtures(),
    generateHighlightSearchFixtures(),
    generateFindMatchesFixtures(),
    generateMapScrollPositionFixtures(),
    generateFindNearestMatchFixtures(),
    generateFormatStatusBarFixtures(),
    generateWordBoundaryFixtures(),
    generateHandleSearchKeyFixtures(),
    generateMapToSourceLineFixtures(),
  ]);
  console.log("  ✓ pager");

  await generateBrowseFixtures();
  console.log("  ✓ browse");

  console.log("Done.");
}
