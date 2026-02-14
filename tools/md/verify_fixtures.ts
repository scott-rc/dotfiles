/**
 * Verify that generated fixtures match the current implementation.
 *
 * Run: deno task verify-fixtures
 *
 * Reads all fixture files and re-runs the functions, asserting
 * that outputs match. Exits non-zero on any mismatch.
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

const FIXTURE_DIR = new URL("./fixtures/", import.meta.url);

function fixtureUrl(path: string): URL {
  return new URL(path, FIXTURE_DIR);
}

async function readFixtureJSON(path: string): Promise<unknown> {
  return JSON.parse(await Deno.readTextFile(fixtureUrl(path)));
}

let passed = 0;
let failed = 0;

function assert(
  label: string,
  actual: unknown,
  expected: unknown,
): void {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    passed++;
  } else {
    failed++;
    console.error(`FAIL: ${label}`);
    console.error(`  expected: ${e}`);
    console.error(`  actual:   ${a}`);
  }
}

// ── Rendering Fixtures ───────────────────────────────────

async function verifyRenderingFixtures() {
  const entries: Deno.DirEntry[] = [];
  for await (const entry of Deno.readDir(fixtureUrl("rendering/"))) {
    if (entry.name.endsWith(".md")) entries.push(entry);
  }

  for (const entry of entries) {
    const name = entry.name.replace(/\.md$/, "");
    const input = await Deno.readTextFile(fixtureUrl(`rendering/${name}.md`));
    const expectedFile = await Deno.readTextFile(
      fixtureUrl(`rendering/${name}.expected.txt`),
    );
    const rendered = await renderMarkdown(input, { width: 60 });
    const actual = stripAnsi(rendered);
    assert(`rendering/${name}`, actual, expectedFile);
  }
}

// ── Wrapping Fixtures ────────────────────────────────────

async function verifyWrappingFixtures() {
  const data = await readFixtureJSON("wrapping/word-wrap.json") as {
    wordWrap: { name: string; input: string; params: { width: number; indent?: string }; expected: string }[];
    stripAnsi: { name: string; input: string; expected: string }[];
    visibleLength: { name: string; input: string; expected: number }[];
  };

  for (const t of data.stripAnsi) {
    assert(`stripAnsi: ${t.name}`, stripAnsi(t.input), t.expected);
  }

  for (const t of data.visibleLength) {
    assert(`visibleLength: ${t.name}`, visibleLength(t.input), t.expected);
  }

  for (const t of data.wordWrap) {
    const actual = wordWrap(t.input, t.params.width, t.params.indent);
    assert(`wordWrap: ${t.name}`, actual, t.expected);
  }
}

// ── Pager Fixtures ───────────────────────────────────────

async function verifyPagerFixtures() {
  // parseKey
  const parseKeyCases = await readFixtureJSON("pager/parse-key.json") as {
    name: string;
    input: number[];
    expected: unknown;
  }[];
  for (const t of parseKeyCases) {
    assert(
      `parseKey: ${t.name}`,
      parseKey(new Uint8Array(t.input)),
      t.expected,
    );
  }

  // truncateLine
  const truncCases = await readFixtureJSON("pager/truncate-line.json") as {
    name: string;
    input: string;
    params: { maxWidth: number };
    expected: string;
  }[];
  for (const t of truncCases) {
    assert(
      `truncateLine: ${t.name}`,
      truncateLine(t.input, t.params.maxWidth),
      t.expected,
    );
  }

  // highlightSearch
  const hlCases = await readFixtureJSON("pager/highlight-search.json") as {
    name: string;
    input: string;
    params: { query: string };
    expected: string;
  }[];
  for (const t of hlCases) {
    assert(
      `highlightSearch: ${t.name}`,
      highlightSearch(t.input, t.params.query),
      t.expected,
    );
  }

  // findMatches
  const fmCases = await readFixtureJSON("pager/find-matches.json") as {
    name: string;
    input: { lines: string[]; query: string };
    expected: number[];
  }[];
  for (const t of fmCases) {
    assert(
      `findMatches: ${t.name}`,
      findMatches(t.input.lines, t.input.query),
      t.expected,
    );
  }

  // mapScrollPosition
  const mspCases = await readFixtureJSON(
    "pager/map-scroll-position.json",
  ) as {
    name: string;
    input: { oldTopLine: number; oldLineCount: number; newLineCount: number };
    expected: number;
  }[];
  for (const t of mspCases) {
    assert(
      `mapScrollPosition: ${t.name}`,
      mapScrollPosition(
        t.input.oldTopLine,
        t.input.oldLineCount,
        t.input.newLineCount,
      ),
      t.expected,
    );
  }

  // findNearestMatch
  const fnmCases = await readFixtureJSON(
    "pager/find-nearest-match.json",
  ) as {
    name: string;
    input: { matches: number[]; topLine: number };
    expected: number;
  }[];
  for (const t of fnmCases) {
    assert(
      `findNearestMatch: ${t.name}`,
      findNearestMatch(t.input.matches, t.input.topLine),
      t.expected,
    );
  }

  // formatStatusBar
  const fsbCases = await readFixtureJSON(
    "pager/format-status-bar.json",
  ) as {
    name: string;
    input: { state: StatusBarInput; cols: number };
    expected: string;
  }[];
  for (const t of fsbCases) {
    assert(
      `formatStatusBar: ${t.name}`,
      formatStatusBar(t.input.state, t.input.cols),
      t.expected,
    );
  }

  // wordBoundary
  const wbData = await readFixtureJSON("pager/word-boundary.json") as {
    left: { name: string; input: { text: string; cursor: number }; expected: number }[];
    right: { name: string; input: { text: string; cursor: number }; expected: number }[];
  };
  for (const t of wbData.left) {
    assert(
      `wordBoundaryLeft: ${t.name}`,
      wordBoundaryLeft(t.input.text, t.input.cursor),
      t.expected,
    );
  }
  for (const t of wbData.right) {
    assert(
      `wordBoundaryRight: ${t.name}`,
      wordBoundaryRight(t.input.text, t.input.cursor),
      t.expected,
    );
  }

  // handleSearchKey
  const hskCases = await readFixtureJSON(
    "pager/handle-search-key.json",
  ) as {
    name: string;
    state: Partial<PagerState>;
    key: { type: string; char?: string };
    expected: Partial<PagerState>;
  }[];
  for (const t of hskCases) {
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
    handleSearchKey(state, t.key as Parameters<typeof handleSearchKey>[1]);
    for (const [key, val] of Object.entries(t.expected)) {
      assert(
        `handleSearchKey: ${t.name} → ${key}`,
        (state as Record<string, unknown>)[key],
        val,
      );
    }
  }

  // mapToSourceLine
  const mtslCases = await readFixtureJSON(
    "pager/map-to-source-line.json",
  ) as {
    name: string;
    input: { topLine: number; renderedLineCount: number; rawContent: string };
    expected: number;
  }[];
  for (const t of mtslCases) {
    assert(
      `mapToSourceLine: ${t.name}`,
      mapToSourceLine(
        t.input.topLine,
        t.input.renderedLineCount,
        t.input.rawContent,
      ),
      t.expected,
    );
  }
}

// ── Browse Fixtures ──────────────────────────────────────

async function verifyBrowseFixtures() {
  const sqCases = await readFixtureJSON("browse/shell-quote.json") as {
    name: string;
    input: string;
    expected: string;
  }[];
  for (const t of sqCases) {
    assert(`shellQuote: ${t.name}`, shellQuote(t.input), t.expected);
  }

  const bfcCases = await readFixtureJSON("browse/build-find-cmd.json") as {
    name: string;
    input: { dir: string; template?: string };
    expected: string;
  }[];
  for (const t of bfcCases) {
    assert(
      `buildFindCmd: ${t.name}`,
      buildFindCmd(t.input.dir, t.input.template),
      t.expected,
    );
  }

  const bpcCases = await readFixtureJSON("browse/build-pick-cmd.json") as {
    name: string;
    input: { template?: string };
    expected: string;
  }[];
  for (const t of bpcCases) {
    assert(
      `buildPickCmd: ${t.name}`,
      buildPickCmd(t.input.template),
      t.expected,
    );
  }

  const psCases = await readFixtureJSON("browse/parse-selection.json") as {
    name: string;
    input: string;
    expected: string | null;
  }[];
  for (const t of psCases) {
    assert(
      `parseSelection: ${t.name}`,
      parseSelection(t.input),
      t.expected,
    );
  }

  const spCases = await readFixtureJSON("browse/should-page.json") as {
    name: string;
    input: {
      noPager: boolean;
      isTTY: boolean;
      contentLines: number;
      terminalRows: number;
      browsing: boolean;
    };
    expected: boolean;
  }[];
  for (const t of spCases) {
    assert(`shouldPage: ${t.name}`, shouldPage(t.input), t.expected);
  }
}

// ── Main ─────────────────────────────────────────────────

if (import.meta.main) {
  console.log("Verifying fixtures...");

  await verifyRenderingFixtures();
  console.log("  ✓ rendering");

  await verifyWrappingFixtures();
  console.log("  ✓ wrapping");

  await verifyPagerFixtures();
  console.log("  ✓ pager");

  await verifyBrowseFixtures();
  console.log("  ✓ browse");

  console.log(`\n${passed} passed, ${failed} failed`);
  if (failed > 0) Deno.exit(1);
}
