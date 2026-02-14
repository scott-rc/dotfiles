import { assertEquals } from "@std/assert";
import { stripAnsi, visibleLength } from "./wrap.ts";
import {
  findMatches,
  findNearestMatch,
  formatStatusBar,
  handleSearchKey,
  highlightSearch,
  type Key,
  mapScrollPosition,
  mapToSourceLine,
  type PagerState,
  parseKey,
  renderStatusBar,
  type StatusBarInput,
  truncateLine,
  wordBoundaryLeft,
  wordBoundaryRight,
} from "./pager.ts";

const FIXTURE_DIR = new URL("./fixtures/pager/", import.meta.url);

async function loadJSON(name: string) {
  return JSON.parse(await Deno.readTextFile(new URL(name, FIXTURE_DIR)));
}

// ── parseKey (from fixtures) ─────────────────────────────

const parseKeyCases = await loadJSON("parse-key.json");

for (const t of parseKeyCases) {
  Deno.test(`parseKey: ${t.name}`, () => {
    assertEquals(parseKey(new Uint8Array(t.input)), t.expected);
  });
}

// ── truncateLine (from fixtures) ─────────────────────────

const truncCases = await loadJSON("truncate-line.json");

for (const t of truncCases) {
  Deno.test(`truncateLine: ${t.name}`, () => {
    assertEquals(truncateLine(t.input, t.params.maxWidth), t.expected);
  });
}

// Additional truncateLine assertions not in fixtures

Deno.test("truncateLine: preserves ANSI codes before truncation point", () => {
  const line = "\x1b[1mhello world\x1b[0m";
  const result = truncateLine(line, 6);
  assertEquals(result.includes("\x1b[1m"), true);
  assertEquals(stripAnsi(result), "hello…");
});

Deno.test("truncateLine: ANSI codes don't count toward width", () => {
  const line = "\x1b[31mhi\x1b[0m";
  assertEquals(truncateLine(line, 10), line);
});

// ── highlightSearch (from fixtures) ──────────────────────

const hlCases = await loadJSON("highlight-search.json");

for (const t of hlCases) {
  Deno.test(`highlightSearch: ${t.name}`, () => {
    assertEquals(highlightSearch(t.input, t.params.query), t.expected);
  });
}

// Additional ANSI-aware assertions

Deno.test("highlightSearch: reverse video codes present on match", () => {
  const result = highlightSearch("Hello World", "hello");
  assertEquals(result.includes("\x1b[7m"), true);
  assertEquals(result.includes("\x1b[27m"), true);
  assertEquals(stripAnsi(result), "Hello World");
});

Deno.test("highlightSearch: multiple matches have multiple reverse pairs", () => {
  const result = highlightSearch("abcabc", "ab");
  const matches = result.match(/\x1b\[7m/g);
  assertEquals(matches?.length, 2);
});

// ── findMatches (from fixtures) ──────────────────────────

const fmCases = await loadJSON("find-matches.json");

for (const t of fmCases) {
  Deno.test(`findMatches: ${t.name}`, () => {
    assertEquals(findMatches(t.input.lines, t.input.query), t.expected);
  });
}

// ── mapToSourceLine (from fixtures) ──────────────────────

const mtslCases = await loadJSON("map-to-source-line.json");

for (const t of mtslCases) {
  Deno.test(`mapToSourceLine: ${t.name}`, () => {
    assertEquals(
      mapToSourceLine(t.input.topLine, t.input.renderedLineCount, t.input.rawContent),
      t.expected,
    );
  });
}

// ── mapScrollPosition (from fixtures) ────────────────────

const mspCases = await loadJSON("map-scroll-position.json");

for (const t of mspCases) {
  Deno.test(`mapScrollPosition: ${t.name}`, () => {
    assertEquals(
      mapScrollPosition(t.input.oldTopLine, t.input.oldLineCount, t.input.newLineCount),
      t.expected,
    );
  });
}

// ── findNearestMatch (from fixtures) ─────────────────────

const fnmCases = await loadJSON("find-nearest-match.json");

for (const t of fnmCases) {
  Deno.test(`findNearestMatch: ${t.name}`, () => {
    assertEquals(findNearestMatch(t.input.matches, t.input.topLine), t.expected);
  });
}

// ── formatStatusBar (from fixtures) ──────────────────────

const fsbCases = await loadJSON("format-status-bar.json");

for (const t of fsbCases) {
  Deno.test(`formatStatusBar: ${t.name}`, () => {
    assertEquals(formatStatusBar(t.input.state, t.input.cols), t.expected);
  });
}

// Additional structural assertions

Deno.test("formatStatusBar: line range info is dimmed", () => {
  const input: StatusBarInput = {
    mode: "normal",
    searchInput: "",
    searchCursor: 0,
    searchMessage: "",
    searchQuery: "",
    searchMatches: [],
    currentMatch: -1,
    topLine: 10,
    lineCount: 50,
    contentHeight: 24,
    filePath: "/path/to/README.md",
  };
  const result = formatStatusBar(input, 60);
  assertEquals(result.includes("\x1b[2m"), true);  // DIM
  assertEquals(result.includes("\x1b[22m"), true); // NO_DIM
});

Deno.test("formatStatusBar: visible width matches cols exactly", () => {
  const input: StatusBarInput = {
    mode: "normal",
    searchInput: "",
    searchCursor: 0,
    searchMessage: "",
    searchQuery: "",
    searchMatches: [],
    currentMatch: -1,
    topLine: 10,
    lineCount: 50,
    contentHeight: 24,
    filePath: "/path/to/README.md",
  };
  for (const cols of [40, 60, 80, 120]) {
    const result = formatStatusBar(input, cols);
    assertEquals(visibleLength(result), cols);
  }
});

// ── renderStatusBar ──────────────────────────────────────

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

Deno.test("renderStatusBar: resets attributes before reverse video", () => {
  const result = renderStatusBar(baseInput(), 60);
  assertEquals(result.startsWith("\x1b[0m\x1b[7m"), true);
});

Deno.test("renderStatusBar: ends with RESET", () => {
  const result = renderStatusBar(baseInput(), 60);
  assertEquals(result.endsWith("\x1b[0m"), true);
});

Deno.test("renderStatusBar: contains formatted status text", () => {
  const result = renderStatusBar(baseInput(), 60);
  const plain = stripAnsi(result);
  assertEquals(plain.includes("README.md"), true);
  assertEquals(plain.includes("TOP"), true);
});

// ── wordBoundary (from fixtures) ─────────────────────────

const wbData = await loadJSON("word-boundary.json");

for (const t of wbData.left) {
  Deno.test(`wordBoundaryLeft: ${t.name}`, () => {
    assertEquals(wordBoundaryLeft(t.input.text, t.input.cursor), t.expected);
  });
}

for (const t of wbData.right) {
  Deno.test(`wordBoundaryRight: ${t.name}`, () => {
    assertEquals(wordBoundaryRight(t.input.text, t.input.cursor), t.expected);
  });
}

// ── handleSearchKey (from fixtures) ──────────────────────

const hskCases = await loadJSON("handle-search-key.json");

for (const t of hskCases) {
  Deno.test(`handleSearchKey: ${t.name}`, () => {
    const state: PagerState = {
      lines: ["hello world", "foo bar", "baz"],
      topLine: 0,
      searchQuery: "",
      searchMatches: [],
      currentMatch: -1,
      mode: "search",
      searchInput: "",
      searchCursor: 0,
      searchMessage: "",
      ...t.state,
    };
    handleSearchKey(state, t.key as Key);
    // deno-lint-ignore no-explicit-any
    const stateAny = state as any;
    for (const [key, val] of Object.entries(t.expected)) {
      assertEquals(
        stateAny[key],
        val,
        `${t.name}: expected ${key} = ${JSON.stringify(val)}, got ${JSON.stringify(stateAny[key])}`,
      );
    }
  });
}
