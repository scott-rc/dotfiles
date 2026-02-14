import { assertEquals } from "@std/assert";
import {
  buildBrowseCmd,
  buildFindCmd,
  buildPickCmd,
  parseSelection,
  shellQuote,
  shouldPage,
} from "./browse.ts";

const FIXTURE_DIR = new URL("./fixtures/browse/", import.meta.url);

async function loadJSON(name: string) {
  return JSON.parse(await Deno.readTextFile(new URL(name, FIXTURE_DIR)));
}

// ── shellQuote (from fixtures) ───────────────────────────

const sqCases = await loadJSON("shell-quote.json");

for (const t of sqCases) {
  Deno.test(`shellQuote: ${t.name}`, () => {
    assertEquals(shellQuote(t.input), t.expected);
  });
}

// ── buildFindCmd (from fixtures) ─────────────────────────

const bfcCases = await loadJSON("build-find-cmd.json");

for (const t of bfcCases) {
  Deno.test(`buildFindCmd: ${t.name}`, () => {
    assertEquals(buildFindCmd(t.input.dir, t.input.template), t.expected);
  });
}

// ── buildPickCmd (from fixtures) ─────────────────────────

const bpcCases = await loadJSON("build-pick-cmd.json");

for (const t of bpcCases) {
  Deno.test(`buildPickCmd: ${t.name}`, () => {
    assertEquals(buildPickCmd(t.input.template), t.expected);
  });
}

// ── buildBrowseCmd ───────────────────────────────────────

Deno.test("buildBrowseCmd: creates default pipeline", () => {
  const cmd = buildBrowseCmd("/docs");
  assertEquals(
    cmd,
    "find '/docs' -type f \\( -name '*.md' -o -name '*.mdx' \\) | fzf",
  );
});

Deno.test("buildBrowseCmd: uses custom commands", () => {
  const cmd = buildBrowseCmd(
    "/docs",
    "find {dir} -name '*.md'",
    "fzf --exact",
  );
  assertEquals(cmd, "find '/docs' -name '*.md' | fzf --exact");
});

// ── parseSelection (from fixtures) ───────────────────────

const psCases = await loadJSON("parse-selection.json");

for (const t of psCases) {
  Deno.test(`parseSelection: ${t.name}`, () => {
    assertEquals(parseSelection(t.input), t.expected);
  });
}

// ── shouldPage (from fixtures) ────────────────────────────

const spCases = await loadJSON("should-page.json");

for (const t of spCases) {
  Deno.test(`shouldPage: ${t.name}`, () => {
    assertEquals(shouldPage(t.input), t.expected);
  });
}
