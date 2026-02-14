import { assertEquals } from "@std/assert";
import { stripAnsi, visibleLength, wordWrap } from "./wrap.ts";

const fixtures = JSON.parse(
  await Deno.readTextFile(
    new URL("./fixtures/wrapping/word-wrap.json", import.meta.url),
  ),
);

// ── stripAnsi (from fixtures) ────────────────────────────

for (const t of fixtures.stripAnsi) {
  Deno.test(`stripAnsi: ${t.name}`, () => {
    assertEquals(stripAnsi(t.input), t.expected);
  });
}

// ── visibleLength (from fixtures) ────────────────────────

for (const t of fixtures.visibleLength) {
  Deno.test(`visibleLength: ${t.name}`, () => {
    assertEquals(visibleLength(t.input), t.expected);
  });
}

// ── wordWrap (from fixtures) ─────────────────────────────

for (const t of fixtures.wordWrap) {
  Deno.test(`wordWrap: ${t.name}`, () => {
    assertEquals(wordWrap(t.input, t.params.width, t.params.indent), t.expected);
  });
}

// ── Additional assertions ────────────────────────────────

Deno.test("wordWrap avoids widow (single word on last line)", () => {
  const result = wordWrap("the quick brown fox jumps over the lazy dog", 20);
  const lines = result.split("\n");
  const lastWords = lines[lines.length - 1].trim().split(/\s+/);
  assertEquals(lastWords.length >= 2, true);
});

Deno.test("wordWrap keeps opening backtick with code content", () => {
  const gray = "\x1b[38;2;139;148;158m";
  const orange = "\x1b[38;2;255;166;87m";
  const reset = "\x1b[39m";
  const codeSpan = `${gray}\`${reset}${orange}code${reset}${gray}\`${reset}`;
  const text = `some text ${codeSpan} end`;
  const result = wordWrap(text, 12);
  const lines = result.split("\n");
  const firstVisible = stripAnsi(lines[0]).trimEnd();
  assertEquals(firstVisible.endsWith("`"), false);
  assertEquals(firstVisible, "some text");
});
